//! Game State Integration HTTP listener.
//!
//! CS2 POSTs JSON to `http://127.0.0.1:<port>/` at a configurable interval
//! (~100ms by default when `hud_fastswitch` / the `heartbeat` tick). We only
//! care about map, phase, scores, and the local player's steam id — see
//! `parse_gsi` for the exact shape.
//!
//! Installing the `.cfg` file is also handled here since both pieces share
//! the port number.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::State as AxState, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::{AppState, SharedSettings};

// Shared marker used by the UI's "GSI connected" pill.
static LAST_SEEN: std::sync::LazyLock<AtomicBool> = std::sync::LazyLock::new(|| AtomicBool::new(false));

pub struct GsiHandle {
    #[allow(dead_code)]
    join: Option<JoinHandle<()>>,
}

#[derive(Clone)]
struct GsiCtx {
    app: AppHandle,
    // Settings currently unused server-side but ready for API forwarding.
    #[allow(dead_code)]
    settings: SharedSettings,
}

pub fn spawn_listener(app: AppHandle, port: u16, settings: SharedSettings) -> GsiHandle {
    let ctx = GsiCtx { app, settings };

    let router = Router::new()
        .route("/", post(gsi_handler))
        .with_state(ctx);

    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let join = tokio::spawn(async move {
        tracing::info!(%addr, "GSI listener starting");
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                if let Err(err) = axum::serve(listener, router).await {
                    tracing::error!(?err, "GSI listener stopped with error");
                }
            }
            Err(err) => {
                tracing::error!(?err, %addr, "Failed to bind GSI port");
            }
        }
    });

    GsiHandle { join: Some(join) }
}

/// Wire format: CS2 posts the whole state object every tick. We parse only
/// the fields we need.
#[derive(Debug, Deserialize)]
struct GsiIncoming {
    #[serde(default)]
    provider: Option<Provider>,
    #[serde(default)]
    map: Option<Map>,
    #[serde(default)]
    round: Option<Round>,
}

#[derive(Debug, Deserialize)]
struct Provider {
    #[serde(rename = "steamid")]
    steam_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Map {
    name: Option<String>,
    mode: Option<String>,
    phase: Option<String>,
    #[serde(default)]
    team_ct: Option<Team>,
    #[serde(default)]
    team_t: Option<Team>,
    #[serde(default)]
    round: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Team {
    score: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Round {
    phase: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GsiUpdate {
    map: Option<String>,
    mode: Option<String>,
    phase: Option<String>,
    ct_score: i32,
    t_score: i32,
    round_number: Option<i32>,
    steam_id64: Option<String>,
    timestamp: u64,
}

async fn gsi_handler(
    AxState(ctx): AxState<GsiCtx>,
    Json(payload): Json<GsiIncoming>,
) -> StatusCode {
    LAST_SEEN.store(true, Ordering::Relaxed);

    let update = GsiUpdate {
        map: payload.map.as_ref().and_then(|m| m.name.clone()),
        mode: payload.map.as_ref().and_then(|m| m.mode.clone()),
        phase: payload
            .map
            .as_ref()
            .and_then(|m| m.phase.clone())
            .or_else(|| payload.round.as_ref().and_then(|r| r.phase.clone())),
        ct_score: payload
            .map
            .as_ref()
            .and_then(|m| m.team_ct.as_ref())
            .and_then(|t| t.score)
            .unwrap_or(0),
        t_score: payload
            .map
            .as_ref()
            .and_then(|m| m.team_t.as_ref())
            .and_then(|t| t.score)
            .unwrap_or(0),
        round_number: payload.map.as_ref().and_then(|m| m.round),
        steam_id64: payload.provider.as_ref().and_then(|p| p.steam_id.clone()),
        timestamp: now_unix_ms(),
    };

    if let Err(err) = ctx.app.emit("gsi:update", &update) {
        tracing::warn!(?err, "Failed to emit gsi:update");
    }

    StatusCode::OK
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ─── Commands ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn gsi_is_connected(_state: State<'_, AppState>) -> Result<bool, String> {
    Ok(LAST_SEEN.load(Ordering::Relaxed))
}

#[tauri::command]
pub async fn gsi_install(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut s = state.settings.lock().await;

    let cfg_dir: PathBuf = s
        .resolve_cfg_dir()
        .ok_or_else(|| "CS2 path not set — pick it in Settings first".to_string())?;

    std::fs::create_dir_all(&cfg_dir).map_err(|e| format!("mkdir cfg: {e}"))?;

    let cfg_path = cfg_dir.join("gamestate_integration_cswatch.cfg");
    let cfg = render_gsi_cfg(s.gsi_port);
    std::fs::write(&cfg_path, cfg).map_err(|e| format!("write cfg: {e}"))?;

    s.gsi_installed = true;
    s.save(&app).map_err(|e| e.to_string())?;

    tracing::info!(path = %cfg_path.display(), "Installed GSI config");
    Ok(())
}

fn render_gsi_cfg(port: u16) -> String {
    // https://developer.valvesoftware.com/wiki/Counter-Strike_2/Game_State_Integration
    format!(
        r#""cswatch Desktop"
{{
  "uri"     "http://127.0.0.1:{port}/"
  "timeout" "5.0"
  "buffer"  "0.1"
  "throttle" "0.1"
  "heartbeat" "10.0"
  "data"
  {{
    "provider"            "1"
    "map"                 "1"
    "round"               "1"
    "player_id"           "1"
    "player_state"        "1"
    "player_match_stats"  "1"
    "allplayers_id"       "1"
    "allplayers_state"    "1"
    "allplayers_match_stats" "1"
  }}
}}
"#
    )
}

// Keep the "unused" State type happy
#[allow(dead_code)]
pub(crate) fn _aliases(_: &Mutex<GsiHandle>, _: Arc<GsiHandle>) {}
