use std::net::Ipv4Addr;
use tokio::sync::RwLock;
use crate::Config;

pub struct AppState {
    pub config: Config,
    pub ip_addr: RwLock<Option<Ipv4Addr>>,
    pub mac_address: RwLock<Option<String>>,
}

