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

## Optional release artifacts

```bash
npm run tauri build
```

Upload `src-tauri/target/release/bundle/dmg/*.dmg` to a GitHub Release tagged `v0.1.0`.

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

GitHub Actions workflow at `.github/workflows/ci.yml` runs on push/PR to `main`:
- `npm ci` + `npm run build`
- `cargo check` in `src-tauri/`

## License

MIT — see [LICENSE](./LICENSE).
