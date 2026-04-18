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
          Roster auto-populates once you join a CS2 match — the GSI feed
          sends all 10 players every tick.
        </p>
        <p
          className="hint"
          style={{ marginTop: "8px", fontSize: "11px", opacity: 0.7 }}
        >
          Still empty after a match starts? Check Settings → GSI is installed,
          and the status pill in the top-right reads{" "}
          <code style={{ color: "var(--gold)" }}>GSI Connected</code>.
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
