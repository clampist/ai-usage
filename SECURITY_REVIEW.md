# Security Review — AI Usage

**Date:** 2026-06-20  
**Scope:** Full codebase (`src-tauri/`, `src/`, config, build)  
**App version:** 0.1.0

This document merges:

1. **Privacy leakage audit** (gstack, 2026-06-20) — source: `.gstack/security-reports/2026-06-20-privacy-leakage-audit.json`
2. **Extended manual review** — Antigravity OAuth, IPC surface, release hygiene (same session)

---

## Executive Summary

AI Usage is a **local-first macOS menu bar app** that reads credentials from the user's machine (Keychain, local JSON, SQLite) and polls **official vendor usage APIs**. It has **no custom backend**, **no telemetry**, and **no remote code loading** in production builds.

| Severity | Privacy audit | Extended review | Combined |
|----------|---------------|-----------------|----------|
| Critical | 0 | 0 | **0** |
| High     | 0 | 0 | **0** |
| Medium   | 6 | 1 (+ updates) | **7** |
| Low      | — | 4 | **4** |
| Info     | — | 5 | **5** |

**Overall posture:** Acceptable for open-source release as a personal productivity tool, with documented privacy trade-offs. No critical vulnerabilities found.

**Remediation since audit:**

| Finding | Status |
|---------|--------|
| CSP disabled (`null`) | ✅ Fixed in `tauri.conf.json` |
| Incomplete `.gitignore` | ✅ Fixed |
| `SECURITY.md` missing | ✅ Added |
| Config dir in docs still said `com.clampist.toolbox` | ✅ Updated to `com.clampist.ai-usage` |

---

## Privacy Leakage Audit (2026-06-20)

> **Source:** `.gstack/security-reports/2026-06-20-privacy-leakage-audit.json`  
> **Mode:** daily · **Scope:** code · **Phases run:** 0, 1, 7, 9, 11, 12, 13, 14

### Attack Surface

| Area | Count / value |
|------|----------------|
| Public endpoints | 0 |
| Authenticated endpoints | 0 |
| Admin endpoints | 0 |
| External API integrations | 3 (Anthropic, OpenAI/Codex, Cursor) |
| Upload handlers | 0 |
| Background jobs | 0 |
| WebSockets | 0 |
| Secret management | macOS Keychain + local files |
| CI workflows (at audit time) | 0 — now has `.github/workflows/ci.yml` |

### Audit Totals

```json
{ "critical": 0, "high": 0, "medium": 6, "tentative": 0 }
```

**Scan stats:** 12 candidates scanned · 4 hard-exclusion filtered · 2 confidence-gate filtered · **6 reported**

**Trend:** First run — no prior report.

### Supply Chain (at audit time)

| Metric | Value |
|--------|-------|
| Lockfile present | yes (`package-lock.json`) |
| Lockfile tracked | no (pre-git) — **now tracked after `git init`** |
| Critical / high CVEs | 0 |
| Install scripts | 0 |
| Skipped | `npm audit` — focused privacy audit |

---

### Finding 1 — No third-party telemetry (positive, rated Medium)

| Field | Value |
|-------|-------|
| **ID** | 1 |
| **Severity** | MEDIUM (informational posture) |
| **Confidence** | 8/10 |
| **Status** | VERIFIED |
| **Phase** | 0 — Architecture Mental Model |
| **Category** | Information Disclosure |
| **Location** | `src-tauri/src/providers/` |

**Description:** Outbound HTTP is limited to `api.anthropic.com`, `chatgpt.com` backend-api, and `api2.cursor.sh`. No analytics, crash reporting, or custom backend.

**Exploit scenario:** Attacker intercepts traffic and sees Bearer tokens only when the app polls official vendor usage endpoints — expected OAuth usage, not exfiltration to an unknown server.

**Impact:** Low external leakage risk from app design; user OAuth tokens traverse the network only to the same vendors already trusted by Claude/Codex/Cursor clients.

**Recommendation:** Document this in README/privacy notice. Optional: pin TLS or log outbound host allowlist in code review checklist.

**Current status:** ✅ Documented in README and SECURITY.md. Antigravity adds `daily-cloudcode-pa.googleapis.com` and `oauth2.googleapis.com` (same trust model).

---

### Finding 2 — Email and plan exposed to WebView via Tauri IPC

| Field | Value |
|-------|-------|
| **ID** | 2 |
| **Severity** | MEDIUM |
| **Confidence** | 9/10 |
| **Status** | VERIFIED |
| **Phase** | 11 — Data Classification |
| **Category** | Information Disclosure |
| **Location** | `src-tauri/src/types.rs:13` |

**Description:** `fetch_usage` returns `AccountStatus` with `account_email` and `plan` to the React frontend. OAuth access tokens stay in Rust and are not serialized.

**Exploit scenario:** If malicious JS runs inside the WebView (XSS, compromised dev server, or tampered frontend bundle), `invoke('fetch_usage')` results containing email/plan/usage could be read or forwarded.

**Impact:** PII and subscription metadata could leak from the local UI layer; OAuth tokens remain in Rust unless combined with another WebView escape.

**Recommendation:** Set a strict CSP in `tauri.conf.json`; keep `show_account_info` default off for shared-screen users; avoid loading remote content in the WebView.

**Current status:** ⚠️ CSP fixed. `show_account_info` still defaults to `true`.

---

### Finding 3 — Content Security Policy disabled

| Field | Value |
|-------|-------|
| **ID** | 3 |
| **Severity** | MEDIUM |
| **Confidence** | 8/10 |
| **Status** | VERIFIED → **RESOLVED** |
| **Phase** | 9 — OWASP Top 10 (A05) |
| **Category** | Security Misconfiguration |
| **Location** | `src-tauri/tauri.conf.json:31` |

**Description:** `app.security.csp` was `null`, so the WebView had no CSP restricting script or connect sources.

**Exploit scenario:** A future change that injects untrusted HTML/JS, or a dev-mode compromise of `localhost:1420`, could execute arbitrary script with access to Tauri IPC and displayed account data.

**Impact:** Increases blast radius of any WebView content injection; does not by itself send data externally.

**Recommendation:** Add `default-src 'self'; script-src 'self'; connect-src ipc: https://ipc.localhost; object-src 'none'` (adjust for Tauri 2 IPC scheme).

**Current status:** ✅ CSP enabled:

```
default-src 'self'; script-src 'self'; connect-src ipc: https://ipc.localhost;
style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost blob: data:;
object-src 'none'
```

---

### Finding 4 — Always-on-top panel displays account email

| Field | Value |
|-------|-------|
| **ID** | 4 |
| **Severity** | MEDIUM |
| **Confidence** | 8/10 |
| **Status** | VERIFIED |
| **Phase** | 11 — Data Classification |
| **Category** | Information Disclosure |
| **Location** | `src/components/AccountCard.tsx:44` |

**Description:** When `show_account_info` is enabled, cards show `plan · email` on an always-on-top, all-workspaces window.

**Exploit scenario:** Shoulder surfing, screen sharing, or screenshots capture work email and plan tier without any network attack.

**Impact:** Local/display privacy exposure for users in open offices or on shared screens.

**Recommendation:** Default `show_account_info` to `false` for privacy-first installs; document the toggle.

**Current status:** ⚠️ Open — documented in README/SECURITY.md; default still `true`.

---

### Finding 5 — Public repo documents local credential paths

| Field | Value |
|-------|-------|
| **ID** | 5 |
| **Severity** | MEDIUM |
| **Confidence** | 9/10 |
| **Status** | VERIFIED |
| **Phase** | 2 — Secrets Archaeology |
| **Category** | Information Disclosure |
| **Location** | `README.md` |

**Description:** Source and README disclose Keychain service names, `~/.codex/auth.json`, Cursor `state.vscdb` paths, and config dir.

**Exploit scenario:** Attacker publishes malware targeting the same paths/keychain entries; no repo secret required, but public code lowers reconnaissance cost.

**Impact:** Does not leak user tokens via GitHub; aids targeted local credential theft if combined with malware on the victim Mac.

**Recommendation:** Accept as open-source tradeoff; avoid committing example outputs with real emails; add SECURITY.md.

**Current status:** ✅ SECURITY.md added. Config path updated to `com.clampist.ai-usage`. Do not commit debug output with real emails.

---

### Finding 6 — Incomplete `.gitignore` before public release

| Field | Value |
|-------|-------|
| **ID** | 6 |
| **Severity** | MEDIUM |
| **Confidence** | 8/10 |
| **Status** | VERIFIED → **RESOLVED** |
| **Phase** | 2 — Secrets Archaeology |
| **Category** | Secrets / release hygiene |
| **Location** | `.gitignore` |

**Description:** `.gitignore` excluded `node_modules` and `dist` but not `src-tauri/target/`, `.omc/`, or macOS artifacts. Build output contained `/Users/clampist/` paths.

**Exploit scenario:** Developer runs `git add -A` or publishes without review; `target/` and `.rustc_info.json` expose local username and toolchain paths on GitHub.

**Impact:** Username and environment fingerprint leaked; not OAuth tokens, but privacy/metadata exposure.

**Recommendation:** Add `src-tauri/target/`, `.omc/`, `*.app`, and `.DS_Store`; run gitleaks before first public push.

**Current status:** ✅ `.gitignore` updated. ⬜ Run gitleaks before first push.

---

## Extended Findings (manual review)

Findings from the same release-prep session, not in the original JSON audit.

### Medium

| ID | Location | Finding | Recommendation |
|----|----------|---------|----------------|
| M7 | `src-tauri/src/providers/antigravity.rs:7-10` | **Embedded Google OAuth client ID + secret** — public pair from Antigravity CLI / Antigravity-Manager, not a user secret. | Document in SECURITY.md (done). Monitor Google OAuth policy. |
| M8 | `src-tauri/src/lib.rs:75-98` | **IPC config writes** accept arbitrary JSON from frontend with minimal validation. | Add bounds on poll interval, window coords, string lengths. |

### Low

| ID | Location | Finding |
|----|----------|---------|
| L1 | `src-tauri/src/keychain.rs:62-70` | `security find-generic-password` via fixed `/usr/bin/security` path — no shell injection. |
| L2 | `src-tauri/src/providers/antigravity.rs` | Refreshed OAuth tokens kept in memory only, not written back to Keychain. |
| L3 | All providers | Unofficial usage APIs may change without notice. |
| L4 | `src-tauri/capabilities/default.json` | Broad Tauri permissions but no filesystem/shell plugins exposed to frontend. |

### Info (positive practices)

| ID | Finding |
|----|---------|
| I1 | No third-party telemetry — vendor APIs only |
| I2 | OAuth tokens never serialized to frontend |
| I3 | reqwest uses rustls |
| I4 | Disabled providers skip credential reads entirely |
| I5 | macOS Accessory activation policy (no Dock icon) |

---

## Architecture & Trust Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│  macOS (user session)                                       │
│  ┌──────────────┐    IPC (Tauri invoke)    ┌─────────────┐  │
│  │ React WebView│ ◄──────────────────────► │ Rust backend│  │
│  │ (bundled)    │   email, plan, usage %   │             │  │
│  └──────────────┘                          │  Keychain   │  │
│                                            │  ~/.codex   │  │
│                                            │  Cursor DB  │  │
│                                            └──────┬──────┘  │
└───────────────────────────────────────────────────┼─────────┘
                                                    │ HTTPS (Bearer tokens)
                    ┌───────────────────────────────┼───────────────────────────────┐
                    ▼                               ▼                               ▼
           api.anthropic.com              chatgpt.com/backend-api          api2.cursor.sh
           daily-cloudcode-pa.googleapis.com (Antigravity)
           oauth2.googleapis.com (token refresh)
```

---

## Secrets & Credential Handling

| Source | What is read | Where it goes |
|--------|--------------|---------------|
| Keychain `Claude Code-credentials` | OAuth access token | `api.anthropic.com` |
| `~/.codex/auth.json` | OAuth access token | `chatgpt.com/backend-api` |
| Cursor `state.vscdb` | Session token | `api2.cursor.sh` |
| Keychain `gemini` / `antigravity` | OAuth access + refresh | Google Cloud Code + OAuth |
| App config dir | Window prefs, aliases | Local disk only |

**Antigravity OAuth client:** Loaded at **build time** from `ANTIGRAVITY_OAUTH_*` env vars (see `.env.example`) — the public desktop OAuth client from the `agy` CLI, not a user credential. No longer embedded in source.

---

## IPC Surface

| Command | Risk |
|---------|------|
| `fetch_usage` | Medium — PII to WebView |
| `get_widget_config` / `save_widget_config_cmd` | Low |
| `get_accounts_config` / `save_accounts_config_cmd` | Low |
| `set_always_on_top`, `set_launch_at_login`, `save_window_position`, `hide_panel` | Low |

No shell execution or arbitrary file read commands exposed to the frontend.

---

## Network Allowlist

| Host | Purpose |
|------|---------|
| `api.anthropic.com` | Claude usage |
| `chatgpt.com` | Codex usage |
| `api2.cursor.sh` | Cursor usage |
| `daily-cloudcode-pa.googleapis.com` | Antigravity quota |
| `oauth2.googleapis.com` | Antigravity token refresh |
| `www.googleapis.com` | Google userinfo (email) |

---

## Pre-Release Checklist

- [x] Comprehensive `.gitignore`
- [x] `SECURITY.md`
- [x] CSP enabled
- [x] Privacy audit findings incorporated in this document
- [ ] Run `gitleaks` before first public push
- [ ] Consider default `show_account_info: false`
- [ ] Apple code signing / notarization (optional)

---

## Conclusion

The privacy audit found **no critical or high issues** — only medium-severity privacy and release-hygiene items. Two are resolved (CSP, `.gitignore`). Remaining items are display privacy defaults and pre-push secret scanning.

For vulnerability reports, see [SECURITY.md](./SECURITY.md).

**Raw audit artifact:** `.gstack/security-reports/2026-06-20-privacy-leakage-audit.json`
