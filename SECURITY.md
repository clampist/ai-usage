# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |

## Reporting a Vulnerability

If you discover a security issue, please **do not open a public GitHub issue** for sensitive reports.

Email or DM the maintainer with:

- Description of the issue
- Steps to reproduce
- Impact assessment (what data could be exposed)
- Your environment (macOS version, app version)

We aim to acknowledge reports within 7 days.

## Scope

**In scope:**

- Credential handling (Keychain, local files)
- Network traffic and token exposure
- Tauri IPC / WebView isolation
- Local config file handling

**Out of scope:**

- Vulnerabilities in third-party CLI tools (Claude Code, Codex, Cursor, Antigravity)
- Vendor API changes or rate limits
- macOS Keychain access prompts (expected OS behavior)
- Issues requiring physical access to an unlocked Mac

## How This App Handles Secrets

AI Usage reads OAuth tokens **already stored on your Mac** by other applications. Tokens are:

- Used only to call **official vendor HTTPS APIs**
- Kept in the **Rust backend** — not sent to the React UI
- **Never** sent to any server operated by this project (there is none)

The app does **not** collect telemetry or crash reports.

## Antigravity OAuth Client

The Antigravity provider refreshes tokens using the **public desktop OAuth client** from the Antigravity CLI (`agy`). These values are **not committed to the repository** — set `ANTIGRAVITY_OAUTH_CLIENT_ID` and `ANTIGRAVITY_OAUTH_CLIENT_SECRET` in a local `.env` file (see `.env.example`) when building from source.

## Privacy

When "Show account email" is enabled, your email appears alongside the plan tier in an always-on-top window. Disable this in preferences if you share your screen. Plan tier is always shown when available.

Config is stored locally at:

`~/Library/Application Support/com.clampist.ai-usage/`
