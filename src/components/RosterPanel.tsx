import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Player } from "../types";

interface Props {
  players: Player[];
}

export function RosterPanel({ players }: Props) {
  const [paste, setPaste] = useState<string>("");
  const [pasteMsg, setPasteMsg] = useState<string>("");
  const [pasteOpen, setPasteOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  async function submitPaste() {
    if (!paste.trim()) return;
    setSubmitting(true);
    setPasteMsg("");
    try {
      const count = await invoke<number>("roster_paste_status", { text: paste });
      setPasteMsg(`Parsed ${count} player${count === 1 ? "" : "s"}.`);
      setPaste("");
      setTimeout(() => setPasteOpen(false), 900);
    } catch (e) {
      setPasteMsg(String(e));
    } finally {
      setSubmitting(false);
    }
  }
  if (players.length === 0) {
    return (
      <section className="panel roster-panel empty">
        <div className="panel-head">
          <h2>Roster</h2>
          <button
            type="button"
            onClick={() => setPasteOpen((v) => !v)}
            style={{ fontSize: 10 }}
          >
            {pasteOpen ? "Cancel" : "Paste status"}
          </button>
        </div>

        {!pasteOpen && (
          <>
            <p className="hint">
              In a live competitive match CS2 withholds the other players' ids
              from GSI. Use <code style={{ color: "var(--gold)" }}>status</code>{" "}
              in the in-game console, then click{" "}
              <strong>Paste status</strong> to feed it into this app.
            </p>
            <p
              className="hint"
              style={{ marginTop: "8px", fontSize: "11px", opacity: 0.7 }}
            >
              Map / score / round still come through GSI automatically. In
              demo, GOTV or broadcast mode the full roster also auto-populates.
            </p>
          </>
        )}

        {pasteOpen && (
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <p className="hint" style={{ margin: 0 }}>
              In CS2 console run <code style={{ color: "var(--gold)" }}>status</code>,
              copy the full output (Ctrl+A, Ctrl+C in console), paste below.
            </p>
            <textarea
              value={paste}
              onChange={(e) => setPaste(e.target.value)}
              placeholder={`# userid name uniqueid connected ping loss state rate\n#  3 "Player" STEAM_1:0:1234 01:23 45 0 active 196608\n...`}
              rows={8}
              spellCheck={false}
              style={{
                width: "100%",
                fontFamily: "var(--font-mono)",
                fontSize: 11,
                background: "var(--bg)",
                color: "var(--text)",
                border: "1px solid var(--border-strong)",
                padding: 8,
                resize: "vertical",
              }}
            />
            <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <button
                type="button"
                onClick={submitPaste}
                disabled={submitting || !paste.trim()}
              >
                {submitting ? "Parsing…" : "Load roster"}
              </button>
              <button
                type="button"
                onClick={async () => {
                  try {
                    const text = await navigator.clipboard.readText();
                    setPaste(text);
                  } catch {
                    setPasteMsg("Clipboard read failed");
                  }
                }}
                disabled={submitting}
              >
                From clipboard
              </button>
              {pasteMsg && (
                <span
                  className="hint"
                  style={{
                    margin: 0,
                    fontSize: 11,
                    color: pasteMsg.startsWith("Parsed")
                      ? "var(--risk-low)"
                      : "var(--risk-high)",
                  }}
                >
                  {pasteMsg}
                </span>
              )}
            </div>
          </div>
        )}
      </section>
    );
  }

  return (
    <section className="panel roster-panel">
      <div className="panel-head">
        <h2>Roster</h2>
        <span className="count">{players.length} players</span>
      </div>
      <ul className="player-list">
        {players.map((p) => (
          <li key={p.steamId} className="player-row">
            <div className="player-name">{p.name ?? "Unknown"}</div>
            <div className="player-id mono">{p.steamId}</div>
            <div className={`risk risk-${p.risk ?? "unknown"}`}>
              {p.risk ?? "—"}
            </div>
            <div className="winprob">
              {typeof p.winProb === "number"
                ? `${(p.winProb * 100).toFixed(0)}%`
                : "—"}
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
