import type { Player } from "../types";

interface Props {
  players: Player[];
}

export function RosterPanel({ players }: Props) {
  if (players.length === 0) {
    return (
      <section className="panel roster-panel empty">
        <h2>Roster</h2>
        <p className="hint">
          No roster detected yet. To populate this panel:
        </p>
        <ol
          className="hint"
          style={{
            margin: "8px 0 0",
            paddingLeft: "20px",
            lineHeight: "1.7",
          }}
        >
          <li>
            Add <code style={{ color: "var(--gold)" }}>-condebug</code> to CS2
            launch options (Steam → properties)
          </li>
          <li>
            Enable <code style={{ color: "var(--gold)" }}>Console watcher</code>{" "}
            in Settings tab
          </li>
          <li>
            In-game console, type <code style={{ color: "var(--gold)" }}>status</code>
          </li>
        </ol>
        <p
          className="hint"
          style={{ marginTop: "8px", fontSize: "11px", opacity: 0.7 }}
        >
          Roster will auto-populate once a match starts. Manual{" "}
          <code>status</code> commands now flush after 1.5s of quiet.
        </p>
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
