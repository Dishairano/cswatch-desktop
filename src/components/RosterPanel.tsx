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
          No roster detected yet. Players will appear once a match starts or
          when the console log shows a `status` command.
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
