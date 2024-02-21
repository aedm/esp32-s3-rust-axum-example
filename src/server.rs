use axum::{extract::State, routing::*, Json, Router};
use log::info;
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use std::net::Ipv4Addr;
use std::sync::atomic::AtomicIsize;
use tokio::sync::RwLock;
use crate::wifi::WifiState;

// Shared state that all Axum handlers can access
struct SharedState {
    pub counter: AtomicIsize,
    pub wifi_state: Arc<WifiState>,
}

// Starts the server. This function only returns in case of an error.
pub async fn run_server(wifi_state: Arc<WifiState>) -> anyhow::Result<()> {
    let state = Arc::new(SharedState {
        counter: AtomicIsize::new(0),
        wifi_state,
    });

    let addr = "0.0.0.0:80".parse::<SocketAddr>()?;
    let app = Router::new()
        .route("/", get(move || async { "Hello!" }))
        .route("/state", get(get_state))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API server listening on {addr:?}");
    Ok(axum::serve(listener, app.into_make_service()).await?)
}

// Handler for the /state route
async fn get_state(State(state): State<Arc<SharedState>>) -> Json<Value> {
    let ip = state.wifi_state.ip_addr().await;
    let mac = state.wifi_state.mac_address.clone();
    let counter = state.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Json(json!({
        "message": "Hello from ESP32!",
        "free_heap": unsafe { esp_idf_sys::esp_get_free_heap_size() },
        "ip_address": ip,
        "mac_address": mac,
        "counter": counter,
    }))
}
