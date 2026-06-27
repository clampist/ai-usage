# GitHub Release Checklist

Use this before the first public push to `github.com/clampist/ai-usage`.

## Repository setup

1. Create an empty GitHub repo named **`ai-usage`** (no README/license — this repo includes them).
2. From this directory:

```bash
git add .
git status   # verify: no node_modules/, target/, .omc/, real emails
git commit -m "Initial release: AI Usage macOS menu bar widget"
git remote add origin git@github.com:clampist/ai-usage.git
git push -u origin main
```

3. On GitHub: **Settings → General → Description**  
   `macOS menu bar widget for AI coding tool usage quotas (Claude, Codex, Cursor, Antigravity)`

4. Add topics: `macos`, `tauri`, `claude-code`, `codex`, `cursor`, `antigravity`, `usage`, `menu-bar`

## GitHub Actions release

Repository secrets (Settings → Secrets and variables → Actions):

| Secret | Purpose |
|--------|---------|
| `ANTIGRAVITY_OAUTH_CLIENT_ID` | Antigravity CLI public OAuth client ID (build-time) |
| `ANTIGRAVITY_OAUTH_CLIENT_SECRET` | Antigravity CLI public OAuth client secret (build-time) |

### Automatic release (tag push)

```bash
git tag v0.1.0
git push origin v0.1.0
```

The [Release workflow](.github/workflows/release.yml) builds a macOS `.dmg` and publishes it to [GitHub Releases](https://github.com/clampist/ai-usage/releases).

### Manual release (Actions tab)

1. Go to **Actions → Release → Run workflow**
2. Enter tag (e.g. `v0.1.0`)
3. Download the `.dmg` from the new release when the job completes

## Local release build

```bash
cp .env.example .env   # fill ANTIGRAVITY_OAUTH_* 
npm run tauri build
```

Artifact: `src-tauri/target/release/bundle/dmg/*.dmg`

## Pre-push security scan

```bash
# if gitleaks is installed
gitleaks detect --source . --verbose
```

## Rename summary (from internal `toolbox`)

| Item | Old | New |
|------|-----|-----|
| npm package | `toolbox` | `ai-usage` |
| Rust crate | `toolbox` / `toolbox_lib` | `ai-usage` / `ai_usage_lib` |
| Bundle ID | `com.clampist.toolbox` | `com.clampist.ai-usage` |
| Config dir | `com.clampist.toolbox` | `com.clampist.ai-usage` (auto-migrates if old exists) |
| Product name | AI Usage | AI Usage (unchanged) |

## CI

GitHub Actions:

- **CI** (`.github/workflows/ci.yml`) — runs on push/PR to `main`: `npm build` + `cargo check` (uses Antigravity OAuth secrets)
- **Release** (`.github/workflows/release.yml`) — runs on `v*` tag push or manual dispatch; publishes macOS `.dmg` to GitHub Releases

## License

MIT — see [LICENSE](./LICENSE).
