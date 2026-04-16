//! Tails CS2's `console.log` (written when the user adds `-condebug` to their
//! launch options) to extract the 10-player roster from `status` output and
//! capture new match share codes.
//!
//! We only ever open the file for reading. No injection, no hooks.

use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, State};
use tokio::task::JoinHandle;

use crate::sharecode;
use crate::AppState;

pub struct ConsoleHandle {
    pub stop: Arc<AtomicBool>,
    #[allow(dead_code)]
    pub join: JoinHandle<()>,
}

pub fn spawn(app: AppHandle, path: PathBuf) -> ConsoleHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();

    let join = tokio::spawn(async move {
        if let Err(err) = run_watcher(app, path, stop_clone).await {
            tracing::warn!(?err, "Console watcher exited with error");
        }
    });

    ConsoleHandle { stop, join }
}

async fn run_watcher(
    app: AppHandle,
    path: PathBuf,
    stop: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    tracing::info!(path = %path.display(), "Console watcher started");

    // Open at end-of-file so we don't re-parse historical state.
    let mut last_size: u64 = 0;

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        // File may not exist yet (CS2 not running). Re-poll each second.
        let meta = match tokio::fs::metadata(&path).await {
            Ok(m) => m,
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let size = meta.len();

        // Log rotation / truncation detection.
        if size < last_size {
            tracing::debug!("console.log truncated — resetting offset");
            last_size = 0;
        }

        if size > last_size {
            if let Err(err) = read_new_lines(&app, &path, &mut last_size).await {
                tracing::warn!(?err, "Failed to read console.log tail");
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

async fn read_new_lines(
    app: &AppHandle,
    path: &PathBuf,
    offset: &mut u64,
) -> anyhow::Result<()> {
    // Run the blocking file I/O on a dedicated thread.
    let path = path.clone();
    let start = *offset;
    let (new_offset, lines) = tokio::task::spawn_blocking(move || -> anyhow::Result<(u64, Vec<String>)> {
        let mut f = std::fs::File::open(&path)?;
        f.seek(SeekFrom::Start(start))?;
        let mut reader = BufReader::new(&f);
        let mut lines = Vec::new();
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                break;
            }
            lines.push(line.clone());
        }
        let pos = reader.stream_position()?;
        Ok((pos, lines))
    })
    .await??;

    *offset = new_offset;

    let mut roster: Vec<(String, String)> = Vec::new();
    let mut sharecodes: Vec<String> = Vec::new();

    for raw in lines {
        // Share codes look like `CSGO-xxxxx-xxxxx-xxxxx-xxxxx-xxxxx`.
        for cap in sharecode::SHARECODE_REGEX.captures_iter(&raw) {
            if let Some(m) = cap.get(0) {
                sharecodes.push(m.as_str().to_string());
            }
        }

        // `status` output lines look like:
        //   # 3 "playerName" STEAM_1:0:12345 01:23 45 0 active 64
        if let Some((steam_id, name)) = sharecode::parse_status_line(&raw) {
            roster.push((steam_id, name));
        }

        // A blank `status` block ends with "# end status" or "#end" — when we
        // hit it, flush the accumulated roster to the frontend.
        if raw.contains("#end") || raw.contains("# end") {
            if !roster.is_empty() {
                emit_roster(app, std::mem::take(&mut roster));
            }
        }
    }

    if !roster.is_empty() {
        emit_roster(app, roster);
    }

    for code in sharecodes {
        tracing::info!(%code, "Captured share code from console");
        // TODO: POST to cswatch.gg /api/desktop/sharecode
        let _ = app.emit("sharecode:seen", &code);
    }

    Ok(())
}

fn emit_roster(app: &AppHandle, entries: Vec<(String, String)>) {
    #[derive(serde::Serialize)]
    struct Player<'a> {
        #[serde(rename = "steamId")]
        steam_id: String,
        name: &'a str,
    }

    #[derive(serde::Serialize)]
    struct RosterUpdate<'a> {
        #[serde(rename = "matchId")]
        match_id: Option<&'a str>,
        players: Vec<Player<'a>>,
        source: &'a str,
    }

    let players: Vec<Player> = entries
        .iter()
        .map(|(id, name)| Player {
            steam_id: id.clone(),
            name: name.as_str(),
        })
        .collect();

    let payload = RosterUpdate {
        match_id: None,
        players,
        source: "console",
    };

    if let Err(err) = app.emit("roster:update", &payload) {
        tracing::warn!(?err, "Failed to emit roster:update");
    }
}

// ─── Commands ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn console_watcher_toggle(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<crate::settings::Settings, String> {
    let mut s = state.settings.lock().await;
    s.console_watcher_enabled = !s.console_watcher_enabled;
    s.save(&app).map_err(|e| e.to_string())?;

    let mut handle = state.console.lock().await;
    if s.console_watcher_enabled {
        let path = s
            .resolve_console_log_path()
            .ok_or_else(|| "CS2 path not set".to_string())?;
        if handle.is_none() {
            *handle = Some(spawn(app.clone(), path));
        }
    } else if let Some(h) = handle.take() {
        h.stop.store(true, Ordering::Relaxed);
    }

    Ok(s.clone())
}
