# AI Usage

[![CI](https://github.com/clampist/ai-usage/actions/workflows/ci.yml/badge.svg)](https://github.com/clampist/ai-usage/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A macOS menu bar widget that shows **usage quotas and reset timers** for AI coding tools — Claude Code, Codex, Cursor, and Antigravity CLI (`agy`).

![macOS](https://img.shields.io/badge/platform-macOS-lightgrey)

## Features

- **Claude Code** — default Keychain credential, plan + session/weekly limits
- **Codex** — active account usage and banked resets from local state
- **Cursor** — enterprise/pro usage via `api2.cursor.sh`
- **Antigravity** — weekly quota groups (Gemini vs Claude/GPT) via Google Cloud Code API
- Live countdown timers until quota resets
- Always-on-top transparent floating panel
- Per-provider enable/disable toggles
- Configurable poll interval and account aliases

## Screenshots

The panel runs from the **menu bar** (not the Dock). Left-click the tray icon to show/hide; right-click for Refresh / Quit.

## Requirements

- macOS 12+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Node.js 18+

You must already be signed in to the tools you want to monitor (Claude Code, Codex.app, Cursor, or `agy`).

## Install

### From source

```bash
git clone https://github.com/clampist/ai-usage.git
cd ai-usage
npm install
npm run tauri build
```

The `.app` bundle is at `src-tauri/target/release/bundle/macos/AI Usage.app`.

### Gatekeeper (unsigned builds)

The app is not Apple-notarized. After copying to `/Applications`, macOS may block launch. Run once:

```bash
xattr -cr "/Applications/AI Usage.app"
```

Or right-click → **Open** → **Open** on first launch.

## Development

```bash
npm install
npm run tauri dev
```

Debug provider output without the UI:

```bash
cargo run --example fetch_usage --manifest-path src-tauri/Cargo.toml
cargo run --example debug_antigravity --manifest-path src-tauri/Cargo.toml
```

### Antigravity OAuth (build-time)

Antigravity token refresh needs the **public Antigravity CLI desktop OAuth client** (same values embedded in the `agy` binary — not a user secret). These are **not stored in source**; they are injected at compile time:

```bash
cp .env.example .env
# Fill ANTIGRAVITY_OAUTH_CLIENT_ID and ANTIGRAVITY_OAUTH_CLIENT_SECRET in .env
npm run tauri build
```

Without `.env`, the app builds but Antigravity refresh returns a configuration error until you rebuild with credentials set. CI uses placeholder values for `cargo check` only.

## Usage

| Action | How |
|--------|-----|
| Show/hide panel | Left-click menu bar icon |
| Refresh now | Right-click → Refresh Now |
| Quit | Right-click → Quit |
| Move panel | Drag title bar or empty card area |
| Preferences | ⚙ button in panel |

## Configuration

Stored at `~/Library/Application Support/com.clampist.ai-usage/`:

| File | Purpose |
|------|---------|
| `widget.json` | Window position, poll interval, provider toggles, always-on-top |
| `accounts.json` | Claude account aliases and hidden accounts |

### Provider toggles (`widget.json`)

```json
{
  "enable_claude": true,
  "enable_codex": true,
  "enable_cursor": false,
  "enable_antigravity": false,
  "show_account_info": true,
  "poll_interval_seconds": 600
}
```

Claude and Codex are enabled by default. Cursor and Antigravity are off until you opt in.

### Claude aliases (`accounts.json`)

```json
{
  "claude": {
    "autoDiscover": true,
    "aliases": {
      "default": "Personal"
    },
    "hidden": []
  }
}
```

## How credentials are read

| Tool | Source |
|------|--------|
| Claude Code | Keychain `Claude Code-credentials` (+ macOS username), fallback `~/.claude/.credentials.json` |
| Codex | `~/.codex/auth.json` |
| Cursor | `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb` |
| Antigravity | Keychain service `gemini`, account `antigravity` |

Tokens are sent **only** to the official vendor HTTPS APIs. There is no analytics, telemetry, or custom backend. See [SECURITY.md](./SECURITY.md) and [SECURITY_REVIEW.md](./SECURITY_REVIEW.md).

## Privacy

When **Show account email** is enabled, your email appears alongside the plan tier in an always-on-top window. Turn this off in preferences if you share your screen. Plan tier is always shown when available.

## Unofficial APIs

Usage endpoints are **unofficial** and may change without notice. This app is not affiliated with Anthropic, OpenAI, Cursor, or Google.

## License

MIT — see [LICENSE](./LICENSE).

## Contributing

Issues and PRs welcome. Run `npm run build` and `cargo check --manifest-path src-tauri/Cargo.toml` before submitting.
