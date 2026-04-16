# cswatch Desktop

CS2 companion app for [cswatch.gg](https://cswatch.gg). Shows your match roster with risk scores and win probability while you play — without ever touching the game process.

## How it works

- **Game State Integration (GSI)** — the app runs a tiny HTTP listener on `127.0.0.1`. CS2 posts map/score/local-player JSON to it every few hundred milliseconds. This is Valve's official, sanctioned API.
- **Console log watcher** — optional. Tails `console.log` (requires `-condebug` in launch options) to parse `status` output for the full 10-player roster.
- **Share code forwarding** — any new share code that scrolls through the console is pushed to cswatch.gg for instant scoring.

**Nothing attaches to, injects into, or reads memory from `cs2.exe`.** Everything is file-based or uses Valve's public APIs, so VAC has zero quarrel with it.

## Dev

```bash
npm install
npm run tauri:dev
```

Requires:
- Node 20+
- Rust stable
- Windows SDK (for Windows builds)

## Build (Windows)

```bash
npm run tauri:build
```

Installers land in `src-tauri/target/release/bundle/nsis/`.

## Releases

Builds run on GitHub Actions (`.github/workflows/release.yml`) when a `v*` tag is pushed. Artifacts are uploaded to [cswatch-desktop-releases](https://github.com/Dishairano/cswatch-desktop-releases) with a `latest.json` Tauri updater manifest.

## License

MIT — see [LICENSE](./LICENSE).
