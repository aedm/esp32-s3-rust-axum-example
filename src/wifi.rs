use crate::config::Config;
use anyhow::{anyhow, Result};
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    ipv4,
    netif::{self, EspNetif},
    timer::{EspTimerService, Task},
    wifi::{AsyncWifi, EspWifi, WifiDriver},
};
use esp_idf_sys::{self as _};
use log::{info, warn};
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

// Shared state of the Wi-Fi connection.
pub struct WifiState {
    pub mac_address: String,
    pub ssid: String,
    ip_addr: RwLock<Option<Ipv4Addr>>,
}

impl WifiState {
    pub async fn ip_addr(&self) -> Option<Ipv4Addr> {
        *self.ip_addr.read().await
    }
}

// Wrapper around the Wi-Fi connection.
pub struct WifiConnection<'a> {
    pub state: Arc<WifiState>,
    wifi: AsyncWifi<EspWifi<'a>>,
}

impl<'a> WifiConnection<'a> {
    // Initialize the Wi-Fi driver but do not connect yet.
    pub async fn new(
        modem: Modem,
        event_loop: EspEventLoop<System>,
        timer: EspTimerService<Task>,
        default_partition: Option<EspDefaultNvsPartition>,
        config: &Config,
    ) -> Result<Self> {
        info!("Initializing...");

        let wifi_driver = WifiDriver::new(modem, event_loop.clone(), default_partition)?;
        let ipv4_config = ipv4::ClientConfiguration::DHCP(ipv4::DHCPClientSettings::default());
        let net_if = EspNetif::new_with_conf(&netif::NetifConfiguration {
            ip_configuration: ipv4::Configuration::Client(ipv4_config),
            ..netif::NetifConfiguration::wifi_default_client()
        })?;

        // Store the MAC address in the shared wifi state
        let mac = net_if.get_mac()?;
        let mac_address = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );
        let state = Arc::new(WifiState {
            ip_addr: RwLock::new(None),
            mac_address,
            ssid: config.wifi_ssid.to_string(),
        });

        // Wrap the Wi-Fi driver in the async wrapper
        let esp_wifi =
            EspWifi::wrap_all(wifi_driver, net_if, EspNetif::new(netif::NetifStack::Ap)?)?;
        let mut wifi = AsyncWifi::wrap(esp_wifi, event_loop, timer.clone())?;

        // Set the Wi-Fi configuration
        info!("Setting credentials...");
        let client_config = ClientConfiguration {
            ssid: heapless::String::from_str(&config.wifi_ssid)
                .map_err(|_| anyhow!("SSID is too long."))?,
            password: heapless::String::from_str(&config.wifi_pass)
                .map_err(|_| anyhow!("Wifi password is too long."))?,
            ..Default::default()
        };
        wifi.set_configuration(&Configuration::Client(client_config))?;

        info!("Starting...");
        wifi.start().await?;

        info!("Wi-Fi driver started successfully.");
        Ok(Self { state, wifi })
    }

    // Connect to Wi-Fi and stay connected. This function will loop forever.
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        loop {
            info!("Connecting to SSID '{}'...", self.state.ssid);
            if let Err(err) = self.wifi.connect().await {
                warn!("Connection failed: {err:?}");
                self.wifi.disconnect().await?;
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            info!("Acquiring IP address...");
            let timeout = Some(Duration::from_secs(10));
            if let Err(err) = self.wifi.ip_wait_while(|w| w.is_up().map(|s| !s), timeout).await {
                warn!("IP association failed: {err:?}");
                self.wifi.disconnect().await?;
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            let ip_info = self.wifi.wifi().sta_netif().get_ip_info();
            *self.state.ip_addr.write().await = ip_info.ok().map(|i| i.ip);
            info!("Connected to '{}': {ip_info:#?}", self.state.ssid);

            // Wait for Wi-Fi to be down
            self.wifi.wifi_wait(|w| w.is_up(), None).await?;
            warn!("Wi-Fi disconnected.");
        }
    }
}
