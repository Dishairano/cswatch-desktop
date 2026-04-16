// Shared types between the React UI and the Rust backend.
// Keep these in sync with the `emit` payloads in src-tauri/src/*.

export interface GsiUpdate {
  map: string | null;
  mode: string | null;
  phase: string | null; // "live" | "warmup" | "intermission" | ...
  ctScore: number;
  tScore: number;
  roundNumber: number | null;
  steamId64: string | null; // local player's steam id
  timestamp: number;
}

export interface Player {
  steamId: string;
  name: string | null;
  // Scores from our backend profile lookup (populated later)
  risk?: "low" | "medium" | "high" | "unknown";
  winProb?: number; // 0..1
  source?: "gsi" | "console" | "sharecode";
}

export interface RosterUpdate {
  matchId: string | null;
  players: Player[];
  source: "gsi" | "console" | "sharecode";
}
