#![warn(clippy::large_futures)]

mod config;
mod state;
mod apiserver;
mod wifi;

use anyhow::{anyhow, bail, Context, Result};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::{gpio::IOPin, prelude::Peripherals};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::gpio, nvs, timer::EspTaskTimerService, wifi::WifiDriver,
};
use esp_idf_sys::{self as _};
use esp_idf_sys::{esp, esp_app_desc};
use std::{net, sync::Arc};
use log::{error, info};
use tokio::sync::RwLock;
use crate::apiserver::run_api_server;
use crate::config::Config;
use crate::state::AppState;
use crate::wifi::WifiConnection;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::io::vfs::initialize_eventfd(1).expect("Failed to initialize eventfd");

    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime")
        .block_on(async move { async_main().await });

    match result {
        Ok(()) => info!("main() finished, reboot."),
        Err(e) => error!("{e:?}"),
    }

    esp_idf_hal::reset::restart();
}

async fn async_main() -> Result<()> {
    info!("Starting async_main.");

    let shared_state = Arc::new(AppState {
        config: Config::default(),
        ip_addr: RwLock::new(None),
        mac_address: RwLock::new(None),
    });
    info!("Configuration:\n{:#?}", shared_state.config);

    let event_loop = EspSystemEventLoop::take()?;
    let timer = EspTaskTimerService::new()?;
    let peripherals = Peripherals::take()?;
    let nvs_default_partition = nvs::EspDefaultNvsPartition::take()?;

    let mut wifi_connection = WifiConnection::new(
        peripherals.modem,
        event_loop,
        timer,
        Some(nvs_default_partition),
        Arc::clone(&shared_state),
    )
    .await?;

    info!("Entering main loop...");

    tokio::try_join!(
        run_api_server(shared_state),
        wifi_connection.stay_connected()
    )?;
    Ok(())
}
