use crate::{AppState, Config};
use anyhow::{anyhow, bail, Context, Result};
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    ipv4,
    netif::{self, EspNetif},
    timer::{EspTimerService, Task},
    wifi::{AsyncWifi, EspWifi, WifiDriver},
};
use esp_idf_sys::{self as _};
use log::*;
use std::str::FromStr;
use std::sync::Arc;
use esp_idf_hal::modem;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use tokio::time::{sleep, Duration};

pub struct WifiConnection<'a> {
    state: Arc<AppState>,
    wifi: AsyncWifi<EspWifi<'a>>,
}

impl<'a> WifiConnection<'a> {
    pub async fn new(
        modem: Modem,
        event_loop: EspEventLoop<System>,
        timer: EspTimerService<Task>,
        default_partition: Option<EspDefaultNvsPartition>,
        state: Arc<AppState>,
    ) -> Result<Self> {
        info!("Initializing...");

        let wifi_driver = WifiDriver::new(
            modem,
            event_loop.clone(),
            default_partition,
        )?;
        let ipv4_config = ipv4::ClientConfiguration::DHCP(ipv4::DHCPClientSettings::default());
        let net_if = EspNetif::new_with_conf(&netif::NetifConfiguration {
            ip_configuration: ipv4::Configuration::Client(ipv4_config),
            ..netif::NetifConfiguration::wifi_default_client()
        })?;

        // Store the MAC address in the shared state
        let mac = net_if.get_mac()?;
        *state.mac_address.write().await = Some(format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        ));

        // Wrap the Wi-Fi driver in the async wrapper
        let esp_wifi =
            EspWifi::wrap_all(wifi_driver, net_if, EspNetif::new(netif::NetifStack::Ap)?)?;
        let mut wifi = AsyncWifi::wrap(esp_wifi, event_loop, timer.clone())?;

        // Set the Wi-Fi configuration
        info!("Setting credentials...");
        let client_config = ClientConfiguration {
            ssid: heapless::String::from_str(&state.config.wifi_ssid)
                .map_err(|_| anyhow!("SSID is too long."))?,
            password: heapless::String::from_str(&state.config.wifi_pass)
                .map_err(|_| anyhow!("Wifi password is too long."))?,
            ..Default::default()
        };
        wifi.set_configuration(&Configuration::Client(client_config))?;

        info!("Starting...");
        wifi.start().await?;

        info!("Wi-Fi driver started successfully.");
        Ok(Self { state, wifi })
    }

    pub async fn stay_connected(&mut self) -> anyhow::Result<()> {
        let mut wifi = &mut self.wifi;
        loop {
            // Wait for Wi-Fi to be down
            wifi.wifi_wait(|w| w.is_up(), None).await?;

            info!("Connecting...");
            if let Err(err) = wifi.connect().await {
                warn!("Connection failed: {err:?}");
                wifi.disconnect().await?;
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            info!("Waiting for IP address...");
            let timeout = Some(Duration::from_secs(10));
            if let Err(err) = wifi.ip_wait_while(|w| w.is_up().map(|s| !s), timeout).await {
                warn!("IP association failed: {err:?}");
                wifi.disconnect().await?;
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            // Store the IP address in the shared state
            if let Ok(ip_info) = wifi.wifi().sta_netif().get_ip_info() {
                info!(
                    "Connected. IP: {:?}, SSID: {}, DNS: {:?}",
                    ip_info.ip, self.state.config.wifi_ssid, ip_info.dns
                );
                *self.state.ip_addr.write().await = Some(ip_info.ip);
            }
        }
    }
}
