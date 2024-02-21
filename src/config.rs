// config.rs

use esp_idf_svc::nvs;
use serde::{Deserialize, Serialize};
use std::net;

const DEFAULT_API_PORT: u16 = 80;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub wifi_ssid: String,
    pub wifi_pass: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: option_env!("API_PORT")
                .unwrap_or("-")
                .parse()
                .unwrap_or(DEFAULT_API_PORT),
            wifi_ssid: option_env!("WIFI_SSID").unwrap_or("internet").into(),
            wifi_pass: option_env!("WIFI_PASS").unwrap_or("password").into(),
        }
    }
}

