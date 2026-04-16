import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Settings {
  cs2Path: string | null;
  gsiPort: number;
  gsiInstalled: boolean;
  consoleLogPath: string | null;
  consoleWatcherEnabled: boolean;
  apiBase: string;
}

export function SettingsPanel() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    invoke<Settings>("settings_get")
      .then(setSettings)
      .catch((e) => setMsg(`Failed to load settings: ${e}`));
  }, []);

  async function pickCs2Path() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir !== "string") return;
    setBusy(true);
    try {
      const updated = await invoke<Settings>("settings_set_cs2_path", {
        path: dir,
      });
      setSettings(updated);
      setMsg("CS2 path saved.");
    } catch (e) {
      setMsg(`Error: ${e}`);
    } finally {
      setBusy(false);
    }
  }

  async function installGsi() {
    if (!settings) return;
    setBusy(true);
    try {
      await invoke("gsi_install");
      const updated = await invoke<Settings>("settings_get");
      setSettings(updated);
      setMsg("GSI config installed. Restart CS2 to pick it up.");
    } catch (e) {
      setMsg(`Error: ${e}`);
    } finally {
      setBusy(false);
    }
  }

  async function toggleConsoleWatcher() {
    if (!settings) return;
    setBusy(true);
    try {
      const updated = await invoke<Settings>("console_watcher_toggle");
      setSettings(updated);
    } catch (e) {
      setMsg(`Error: ${e}`);
    } finally {
      setBusy(false);
    }
  }

  if (!settings) {
    return (
      <section className="panel">
        <h2>Settings</h2>
        <p className="hint">Loading…</p>
      </section>
    );
  }

  return (
    <section className="panel settings-panel">
      <h2>Settings</h2>

      <div className="field">
        <label>CS2 install folder</label>
        <div className="row">
          <code className="path mono">
            {settings.cs2Path ?? "Not set (we'll autodetect)"}
          </code>
          <button disabled={busy} onClick={pickCs2Path}>
            Browse…
          </button>
        </div>
        <p className="hint">
          Used to install the GSI config and tail <code>console.log</code>.
        </p>
      </div>

      <div className="field">
        <label>Game State Integration</label>
        <div className="row">
          <span className="status-pill">
            {settings.gsiInstalled ? "installed" : "not installed"}
          </span>
          <span className="mono">port {settings.gsiPort}</span>
          <button disabled={busy} onClick={installGsi}>
            {settings.gsiInstalled ? "Reinstall" : "Install"}
          </button>
        </div>
        <p className="hint">
          Writes{" "}
          <code>game/csgo/cfg/gamestate_integration_cswatch.cfg</code>. Safe,
          sanctioned Valve API.
        </p>
      </div>

      <div className="field">
        <label>Console log watcher</label>
        <div className="row">
          <span className="status-pill">
            {settings.consoleWatcherEnabled ? "enabled" : "disabled"}
          </span>
          <button disabled={busy} onClick={toggleConsoleWatcher}>
            {settings.consoleWatcherEnabled ? "Disable" : "Enable"}
          </button>
        </div>
        <p className="hint">
          Tails <code>console.log</code> (requires <code>-condebug</code>{" "}
          launch option) to parse <code>status</code> output. Read-only, VAC
          safe.
        </p>
      </div>

      <div className="field">
        <label>API base</label>
        <code className="path mono">{settings.apiBase}</code>
      </div>

      {msg && <div className="toast">{msg}</div>}
    </section>
  );
}
