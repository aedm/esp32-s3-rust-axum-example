mod config;
mod server;
mod wifi;

use crate::config::Config;
use crate::server::run_server;
use crate::wifi::WifiConnection;
use anyhow::Result;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs, timer::EspTaskTimerService};
use log::{error, info};
use std::thread::sleep;

// We can't use #[tokio::main] since we need to initialize eventfd before starting the tokio runtime.
fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::io::vfs::initialize_eventfd(1).expect("Failed to initialize eventfd");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    match rt.block_on(async { async_main().await }) {
        Ok(()) => info!("main() finished, reboot."),
        Err(err) => {
            error!("{err:?}");
            // Let them read the error message before rebooting
            sleep(std::time::Duration::from_secs(3));
        },
    }

    esp_idf_hal::reset::restart();
}

async fn async_main() -> Result<()> {
    info!("Starting async_main.");

    let config = Config::load()?;
    info!("Configuration:\n{config:#?}");

    let event_loop = EspSystemEventLoop::take()?;
    let timer = EspTaskTimerService::new()?;
    let peripherals = Peripherals::take()?;
    let nvs_default_partition = nvs::EspDefaultNvsPartition::take()?;

    // Initialize the network stack, this must be done before starting the server
    let mut wifi_connection = WifiConnection::new(
        peripherals.modem,
        event_loop,
        timer,
        Some(nvs_default_partition),
        &config,
    )
    .await?;

    // Run the server and the wifi keepalive concurrently until one of them fails
    tokio::try_join!(
        run_server(wifi_connection.state.clone()),
        wifi_connection.connect()
    )?;
    Ok(())
}
