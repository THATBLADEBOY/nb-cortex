//! Server status command for the frontend to discover the Hono server port.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::state::AppState;

/// Status of the Hono sidecar server.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ServerStatus {
    /// Whether the Hono server is running and reachable.
    pub running: bool,
    /// The port the Hono server is listening on (if running).
    pub port: Option<u16>,
}

/// Get the current status of the Hono sidecar server.
/// The frontend uses this to discover the Hono port.
#[tauri::command]
#[specta::specta]
pub fn get_server_status(state: State<'_, AppState>) -> ServerStatus {
    let hono_port = state.hono_port.lock().unwrap();
    ServerStatus {
        running: hono_port.is_some(),
        port: *hono_port,
    }
}
