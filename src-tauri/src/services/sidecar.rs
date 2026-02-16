//! Bun sidecar lifecycle management.
//!
//! In dev mode: spawns `bun run server/src/index.ts` directly.
//! In release: uses Tauri sidecar with the compiled `cortex-server` binary.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use tauri::Manager;

use crate::state::AppState;

/// Start the Hono sidecar process.
/// Returns the port the Hono server bound to.
pub fn start_sidecar(
    app: &tauri::AppHandle,
    bridge_port: u16,
    bridge_token: &str,
) -> Result<u16, String> {
    let env_vars = [
        (
            "CORTEX_BRIDGE_URL",
            format!("http://127.0.0.1:{bridge_port}"),
        ),
        ("CORTEX_BRIDGE_TOKEN", bridge_token.to_string()),
        // Port 0 tells the server to pick a random available port
        ("CORTEX_HONO_PORT", "0".to_string()),
    ];

    let mut child = if cfg!(debug_assertions) {
        // Dev mode: run bun directly for hot reload
        log::info!("Starting Hono sidecar in dev mode (bun run)");
        Command::new("bun")
            .arg("run")
            .arg("server/src/index.ts")
            .envs(env_vars)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to spawn bun sidecar: {e}"))?
    } else {
        // Release mode: use compiled sidecar binary
        log::info!("Starting Hono sidecar in release mode");
        let sidecar_path = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {e}"))?
            .join("binaries")
            .join("cortex-server");

        Command::new(sidecar_path)
            .envs(env_vars)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to spawn sidecar binary: {e}"))?
    };

    // Read stdout to find the port the server bound to
    let hono_port = read_hono_port(&mut child)?;

    // Store the child PID for cleanup
    let state = app.state::<AppState>();
    *state.sidecar_pid.lock().unwrap() = Some(child.id());
    *state.hono_port.lock().unwrap() = Some(hono_port);

    // Detach the child so it doesn't get dropped when this function returns.
    // We manage its lifecycle via the stored PID.
    std::mem::forget(child);

    log::info!("Hono sidecar started on port {hono_port}");
    Ok(hono_port)
}

/// Read stdout lines from the child process to find the HONO_PORT marker.
fn read_hono_port(child: &mut Child) -> Result<u16, String> {
    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture sidecar stdout")?;

    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read sidecar output: {e}"))?;
        log::debug!("Sidecar: {line}");

        if let Some(port_str) = line.strip_prefix("HONO_PORT:") {
            let port: u16 = port_str
                .trim()
                .parse()
                .map_err(|e| format!("Failed to parse Hono port: {e}"))?;
            return Ok(port);
        }
    }

    Err("Sidecar exited without reporting a port".to_string())
}

/// Stop the sidecar process by killing the stored PID.
pub fn stop_sidecar(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let pid = state.sidecar_pid.lock().unwrap().take();

    if let Some(pid) = pid {
        log::info!("Stopping sidecar (PID {pid})");

        #[cfg(unix)]
        {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }

        #[cfg(windows)]
        {
            // On Windows, use taskkill
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output();
        }
    }

    *state.hono_port.lock().unwrap() = None;
}
