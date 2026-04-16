//! cswatch Desktop — Tauri 2 backend.
//!
//! Responsibilities:
//!   1. Run a local HTTP server on 127.0.0.1 to receive CS2 Game State
//!      Integration webhooks (map, score, local player).
//!   2. Optionally tail `console.log` (requires `-condebug` launch option) to
//!      parse `status` output and extract the full 10-player roster.
//!   3. Watch for new match share codes (shipped in the same console log) and
//!      POST them to cswatch.gg for instant risk/win-probability scoring.
//!   4. Persist user settings via `tauri-plugin-store`.
//!
//! Importantly, **nothing here attaches to, reads memory from, or otherwise
//! touches the `cs2.exe` process**. Everything uses Valve-sanctioned outputs
//! (GSI webhook, console.log file) so VAC has zero quarrel with us.

mod settings;
mod gsi;
mod console_watcher;
mod sharecode;
mod tray;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use settings::Settings;

pub type SharedSettings = Arc<Mutex<Settings>>;

/// Top-level app state, stashed into Tauri's managed state so commands can
/// reach the settings store, the GSI listener handle, and the console watcher
/// handle from anywhere.
pub struct AppState {
    pub settings: SharedSettings,
    pub gsi: Mutex<gsi::GsiHandle>,
    pub console: Mutex<Option<console_watcher::ConsoleHandle>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,cswatch_desktop_lib=debug")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();

            // Load settings synchronously at startup so the UI can render
            // with the right state immediately.
            let settings = Settings::load_or_default(&handle);
            let settings_arc: SharedSettings = Arc::new(Mutex::new(settings));

            // Spin up the GSI HTTP listener on the configured port.
            let gsi_handle = {
                let app_handle = handle.clone();
                let settings_arc = settings_arc.clone();
                let port = settings_arc.blocking_lock().gsi_port;
                gsi::spawn_listener(app_handle, port, settings_arc)
            };

            // Spin up the console watcher if enabled.
            let console_handle = {
                let s = settings_arc.blocking_lock();
                if s.console_watcher_enabled {
                    match s.resolve_console_log_path() {
                        Some(path) => Some(console_watcher::spawn(handle.clone(), path)),
                        None => {
                            tracing::warn!(
                                "Console watcher enabled but console.log path could not be resolved; skipping"
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            };

            app.manage(AppState {
                settings: settings_arc,
                gsi: Mutex::new(gsi_handle),
                console: Mutex::new(console_handle),
            });

            // Build tray icon + menu.
            tray::install(&handle)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            settings::settings_get,
            settings::settings_set_cs2_path,
            settings::settings_set_api_base,
            gsi::gsi_is_connected,
            gsi::gsi_install,
            console_watcher::console_watcher_toggle,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
