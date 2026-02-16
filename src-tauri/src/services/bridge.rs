//! Axum HTTP bridge server for secure keychain access.
//!
//! Runs on localhost only, authenticated with a per-session bearer token.
//! Only the Hono sidecar should call this — the frontend never sees the
//! bridge port or token.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Shared state for the bridge server.
#[derive(Clone)]
pub struct BridgeState {
    /// The bearer token required for all requests.
    pub token: String,
}

/// Start the axum bridge on a random localhost port.
/// Returns the port it bound to.
pub async fn start_bridge(token: String) -> Result<u16, String> {
    let state = BridgeState {
        token: token.clone(),
    };

    let app = Router::new()
        .route("/api-key/{service}", get(get_api_key))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind bridge server: {e}"))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get bridge port: {e}"))?
        .port();

    log::info!("Axum bridge listening on 127.0.0.1:{port}");

    // Spawn the server as a background task
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            log::error!("Bridge server error: {e}");
        }
    });

    Ok(port)
}

/// Validate the bearer token from request headers.
fn validate_token(headers: &HeaderMap, expected: &str) -> Result<(), StatusCode> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth != format!("Bearer {expected}") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(())
}

/// GET /api-key/:service — reads an API key from the OS keychain.
async fn get_api_key(
    State(state): State<BridgeState>,
    headers: HeaderMap,
    Path(service): Path<String>,
) -> impl IntoResponse {
    if let Err(status) = validate_token(&headers, &state.token) {
        return status.into_response();
    }

    let keychain_service = format!("com.THATBLADEBOY.cortex.api-key.{service}");

    let entry = match keyring::Entry::new(&keychain_service, "api-key") {
        Ok(e) => e,
        Err(e) => {
            log::error!("Failed to create keychain entry for bridge: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match entry.get_password() {
        Ok(key) => Json(serde_json::json!({ "key": key })).into_response(),
        Err(keyring::Error::NoEntry) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            log::error!("Failed to read API key from keychain: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
