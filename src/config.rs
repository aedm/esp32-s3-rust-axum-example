use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Config {
    pub wifi_ssid: &'static str,
    pub wifi_pass: &'static str,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Self {
            wifi_ssid: option_env!("WIFI_SSID")
                .context("WIFI_SSID env variable must be set before building the project.")?,
            wifi_pass: option_env!("WIFI_PASS")
                .context("WIFI_PASS env variable must be set before building the project.")?,
        })
    }
}
