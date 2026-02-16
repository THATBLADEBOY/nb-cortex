//! Shared application state for the Hono sidecar infrastructure.
//!
//! Holds the bridge token, port assignments, and sidecar child process handle.
//! Managed via `tauri::Manager::manage()` and accessed in commands via `tauri::State`.

use std::sync::Mutex;

/// Application state shared across Tauri commands and services.
#[allow(dead_code)]
pub struct AppState {
    /// Per-session bearer token for the axum bridge (UUID v4).
    pub bridge_token: String,
    /// Port the axum bridge is listening on (localhost only).
    pub bridge_port: u16,
    /// Port the Hono server is listening on (localhost only).
    /// Set once the sidecar reports ready.
    pub hono_port: Mutex<Option<u16>>,
    /// Child process ID for the sidecar, used for cleanup.
    pub sidecar_pid: Mutex<Option<u32>>,
}
