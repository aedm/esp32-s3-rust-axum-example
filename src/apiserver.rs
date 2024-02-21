use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::Html, routing::*, Json, Router};
pub use axum_macros::debug_handler;
use core::f32;
use log::*;
use serde::Serialize;
use serde_json::{json, Value};
use std::{net, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::time::{sleep, Duration};

pub async fn run_api_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let addr = "0.0.0.0:80".parse::<SocketAddr>()?;
    let app = Router::new()
        .route("/", get({ move || async { "Hello!" } }))
        .route("/mem", get(get_state))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API server listening on {addr:?}");
    Ok(axum::serve(listener, app.into_make_service()).await?)
}

async fn get_state(State(state): State<Arc<AppState>>) -> Json<Value> {
    let ip = state.ip_addr.read().await.clone();
    let mac = state.mac_address.read().await.clone();
    Json(json!({
        "message": "Hello, World!",
        "free_heap": unsafe { esp_idf_sys::esp_get_free_heap_size() },
        "ip_address": ip,
        "mac_address": mac,
    }))
}
