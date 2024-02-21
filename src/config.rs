#[derive(Debug)]
pub struct Config {
    pub wifi_ssid: &'static str,
    pub wifi_pass: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wifi_ssid: option_env!("WIFI_SSID").unwrap_or("internet"),
            wifi_pass: option_env!("WIFI_PASS").unwrap_or("password"),
        }
    }
}
