import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import type { GsiUpdate, RosterUpdate, Player } from "./types";
import { RosterPanel } from "./components/RosterPanel";
import { ScorePanel } from "./components/ScorePanel";
import { SettingsPanel } from "./components/SettingsPanel";

type View = "live" | "settings";

export default function App() {
  const [view, setView] = useState<View>("live");
  const [gsi, setGsi] = useState<GsiUpdate | null>(null);
  const [roster, setRoster] = useState<Player[]>([]);
  const [gsiConnected, setGsiConnected] = useState(false);
  const [appVersion, setAppVersion] = useState("");

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => setAppVersion(""));
  }, []);

  useEffect(() => {
    const unlistenGsi = listen<GsiUpdate>("gsi:update", (event) => {
      setGsi(event.payload);
      setGsiConnected(true);
    });

    const unlistenRoster = listen<RosterUpdate>("roster:update", (event) => {
      setRoster(event.payload.players);
    });

    // Ask backend if GSI has seen any recent traffic
    invoke<boolean>("gsi_is_connected").then(setGsiConnected).catch(() => {});

    return () => {
      unlistenGsi.then((fn) => fn());
      unlistenRoster.then((fn) => fn());
    };
  }, []);

  return (
    <div className="app">
      <header className="titlebar">
        <div className="brand">
          <span className="logo-dot" />
          <span className="brand-text">cswatch</span>
          <span className="brand-sub">desktop</span>
        </div>
        <nav className="tabs">
          <button
            className={view === "live" ? "tab active" : "tab"}
            onClick={() => setView("live")}
          >
            Live
          </button>
          <button
            className={view === "settings" ? "tab active" : "tab"}
            onClick={() => setView("settings")}
          >
            Settings
          </button>
        </nav>
        <div className="status">
          <span className={gsiConnected ? "dot green" : "dot gray"} />
          <span className="status-label">
            {gsiConnected ? "GSI connected" : "Waiting for CS2"}
          </span>
        </div>
      </header>

      <main className="main">
        {view === "live" && (
          <>
            <ScorePanel gsi={gsi} />
            <RosterPanel players={roster} />
          </>
        )}
        {view === "settings" && <SettingsPanel />}
      </main>

      <footer className="footer">
        <span>
          cswatch.gg desktop{appVersion ? ` · v${appVersion}` : ""} ·{" "}
          <a href="https://cswatch.gg" target="_blank" rel="noreferrer">
            cswatch.gg
          </a>
        </span>
      </footer>
    </div>
  );
}
