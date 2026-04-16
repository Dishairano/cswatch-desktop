import type { GsiUpdate } from "../types";

interface Props {
  gsi: GsiUpdate | null;
}

export function ScorePanel({ gsi }: Props) {
  if (!gsi) {
    return (
      <section className="panel score-panel empty">
        <h2>Live score</h2>
        <p className="hint">
          Waiting for CS2 Game State Integration. Open Settings to install the
          GSI config, then launch CS2.
        </p>
      </section>
    );
  }

  const mapLabel = gsi.map ?? "—";
  const phase = gsi.phase ?? "unknown";
  const round =
    typeof gsi.roundNumber === "number" ? `Round ${gsi.roundNumber}` : "—";

  return (
    <section className="panel score-panel">
      <div className="panel-head">
        <h2>{mapLabel}</h2>
        <span className={`phase phase-${phase}`}>{phase}</span>
      </div>
      <div className="scoreboard">
        <div className="team team-ct">
          <div className="team-label">CT</div>
          <div className="team-score">{gsi.ctScore}</div>
        </div>
        <div className="round-info">{round}</div>
        <div className="team team-t">
          <div className="team-label">T</div>
          <div className="team-score">{gsi.tScore}</div>
        </div>
      </div>
    </section>
  );
}
