//! Persisted user settings. Stored via tauri-plugin-store under
//! `cswatch-settings.json` in the app's data dir.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::AppState;

const SETTINGS_FILE: &str = "cswatch-settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Absolute path to the Counter-Strike 2 install directory, e.g.
    /// `C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive`.
    pub cs2_path: Option<String>,

    /// Port the GSI HTTP listener binds to. Must match the `uri` in the
    /// generated `gamestate_integration_cswatch.cfg`.
    pub gsi_port: u16,

    /// True once we've written the GSI config to `game/csgo/cfg/`.
    pub gsi_installed: bool,

    /// Absolute path to `console.log`. Derived from `cs2_path` when unset.
    pub console_log_path: Option<String>,

    /// Whether to tail the console log for roster/sharecode extraction.
    pub console_watcher_enabled: bool,

    /// cswatch.gg API base. Override for dev/staging.
    pub api_base: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cs2_path: None,
            gsi_port: 3639,
            gsi_installed: false,
            console_log_path: None,
            console_watcher_enabled: false,
            api_base: "https://cswatch.gg".to_string(),
        }
    }
}

impl Settings {
    pub fn load_or_default(app: &AppHandle) -> Self {
        let path = settings_path(app);
        match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|err| {
                tracing::warn!(?err, "Failed to parse settings file, using defaults");
                Settings::default()
            }),
            Err(_) => Settings::default(),
        }
    }

    pub fn save(&self, app: &AppHandle) -> anyhow::Result<()> {
        let path = settings_path(app);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Returns the `console.log` path — either the explicit override or
    /// `<cs2_path>/game/csgo/console.log`.
    pub fn resolve_console_log_path(&self) -> Option<PathBuf> {
        if let Some(p) = &self.console_log_path {
            return Some(PathBuf::from(p));
        }
        let cs2 = self.cs2_path.as_ref()?;
        Some(PathBuf::from(cs2).join("game").join("csgo").join("console.log"))
    }

    pub fn resolve_cfg_dir(&self) -> Option<PathBuf> {
        let cs2 = self.cs2_path.as_ref()?;
        Some(PathBuf::from(cs2).join("game").join("csgo").join("cfg"))
    }
}

fn settings_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    dir.join(SETTINGS_FILE)
}

// ─── Commands ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings.lock().await.clone())
}

#[tauri::command]
pub async fn settings_set_cs2_path(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<Settings, String> {
    let mut s = state.settings.lock().await;
    s.cs2_path = Some(path);
    s.save(&app).map_err(|e| e.to_string())?;
    Ok(s.clone())
}

#[tauri::command]
pub async fn settings_set_api_base(
    app: AppHandle,
    state: State<'_, AppState>,
    base: String,
) -> Result<Settings, String> {
    let mut s = state.settings.lock().await;
    s.api_base = base;
    s.save(&app).map_err(|e| e.to_string())?;
    Ok(s.clone())
}

