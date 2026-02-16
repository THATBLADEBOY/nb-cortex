//! Tauri application library entry point.
//!
//! This module serves as the main entry point for the Tauri application.
//! Command implementations are organized in the `commands` module,
//! and shared types are in the `types` module.

mod bindings;
mod commands;
mod services;
mod state;
mod types;
mod utils;

use std::sync::Mutex;
use tauri::{Emitter, Manager};

// Re-export only what's needed externally
pub use types::DEFAULT_QUICK_PANE_SHORTCUT;

/// Application entry point. Sets up all plugins and initializes the app.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = bindings::generate_bindings();

    // Export TypeScript bindings in debug builds
    #[cfg(debug_assertions)]
    bindings::export_ts_bindings();

    // Build with common plugins
    let mut app_builder = tauri::Builder::default();

    // Single instance plugin must be registered FIRST
    // When user tries to open a second instance, focus the existing window instead
    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }));
    }

    // Window state plugin - saves/restores window position and size
    // Note: Only applies to windows listed in capabilities (main window only, not quick-pane)
    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(tauri_plugin_window_state::StateFlags::all())
                .build(),
        );
    }

    // Updater plugin for in-app updates
    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    app_builder = app_builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                // Use Debug level in development, Info in production
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .targets([
                    // Always log to stdout for development
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    // Log to webview console for development
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                    // Log to system logs on macOS (appears in Console.app)
                    #[cfg(target_os = "macos")]
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                ])
                .build(),
        );

    // macOS: Add NSPanel plugin for native panel behavior
    #[cfg(target_os = "macos")]
    {
        app_builder = app_builder.plugin(tauri_nspanel::init());
    }

    app_builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            log::info!("Application starting up");
            log::debug!(
                "App handle initialized for package: {}",
                app.package_info().name
            );

            // Set up global shortcut plugin (without any shortcuts - we register them separately)
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::Builder;

                app.handle().plugin(Builder::new().build())?;
            }

            // Load saved preferences and register the quick pane shortcut
            #[cfg(desktop)]
            {
                let saved_shortcut = commands::preferences::load_quick_pane_shortcut(app.handle());
                let shortcut_to_register = saved_shortcut
                    .as_deref()
                    .unwrap_or(DEFAULT_QUICK_PANE_SHORTCUT);

                log::info!("Registering quick pane shortcut: {shortcut_to_register}");
                commands::quick_pane::register_quick_pane_shortcut(
                    app.handle(),
                    shortcut_to_register,
                )?;
            }

            // Create the quick pane window (hidden) - must be done on main thread
            if let Err(e) = commands::quick_pane::init_quick_pane(app.handle()) {
                log::error!("Failed to create quick pane: {e}");
                // Non-fatal: app can still run without quick pane
            }

            // Start the Hono sidecar infrastructure
            let app_handle = app.handle().clone();
            start_hono_infrastructure(&app_handle);

            // NOTE: Application menu is built from JavaScript for i18n support
            // See src/lib/menu.ts for the menu implementation

            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .on_window_event(|window, event| {
            // Kill sidecar when the main window is destroyed
            if let tauri::WindowEvent::Destroyed = event {
                if window.label() == "main" {
                    log::info!("Main window destroyed, stopping sidecar");
                    services::sidecar::stop_sidecar(window.app_handle());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Starts the axum bridge and Hono sidecar, then emits `server-ready` to the frontend.
/// Runs on a background thread so it doesn't block app startup.
fn start_hono_infrastructure(app: &tauri::AppHandle) {
    let token = uuid::Uuid::new_v4().to_string();
    log::info!("Generated bridge token for this session");

    let app_handle = app.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for bridge");

        let bridge_port =
            rt.block_on(async { services::bridge::start_bridge(token.clone()).await });

        let bridge_port = match bridge_port {
            Ok(port) => port,
            Err(e) => {
                log::error!("Failed to start axum bridge: {e}");
                return;
            }
        };

        log::info!("Axum bridge started on port {bridge_port}");

        // Manage AppState so commands and sidecar can access it
        app_handle.manage(state::AppState {
            bridge_token: token.clone(),
            bridge_port,
            hono_port: Mutex::new(None),
            sidecar_pid: Mutex::new(None),
        });

        // Start the sidecar
        match services::sidecar::start_sidecar(&app_handle, bridge_port, &token) {
            Ok(hono_port) => {
                log::info!("Hono server ready on port {hono_port}");
                // Emit event so frontend can discover the port
                let _ = app_handle.emit("server-ready", serde_json::json!({ "port": hono_port }));
            }
            Err(e) => {
                log::error!("Failed to start Hono sidecar: {e}");
            }
        }

        // Keep the runtime alive so the axum bridge keeps serving
        rt.block_on(std::future::pending::<()>());
    });
}
