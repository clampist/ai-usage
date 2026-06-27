import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState, type MouseEvent } from "react";
import { AccountCard } from "./components/AccountCard";
import type { AccountsConfig, WidgetConfig, WidgetSnapshot } from "./types";
import "./index.css";

function App() {
  const [snapshot, setSnapshot] = useState<WidgetSnapshot | null>(null);
  const [config, setConfig] = useState<WidgetConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [showPrefs, setShowPrefs] = useState(false);
  const [claudeAlias, setClaudeAlias] = useState("");

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<WidgetSnapshot>("fetch_usage");
      setSnapshot(data);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    invoke<WidgetConfig>("get_widget_config").then(setConfig).catch(console.error);
    invoke<AccountsConfig>("get_accounts_config")
      .then((a) => {
        setClaudeAlias(a.claude.aliases?.default ?? "");
      })
      .catch(console.error);
    refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("refresh-usage", () => {
      refresh();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refresh]);

  useEffect(() => {
    const interval = (config?.poll_interval_seconds ?? 600) * 1000;
    const id = window.setInterval(refresh, interval);
    return () => window.clearInterval(id);
  }, [config?.poll_interval_seconds, refresh]);

  useEffect(() => {
    const savePos = () => {
      invoke("save_window_position").catch(console.error);
    };
    const win = getCurrentWindow();
    const unlisten = win.onMoved(savePos);
    const unlisten2 = win.onResized(savePos);
    return () => {
      unlisten.then((f) => f());
      unlisten2.then((f) => f());
    };
  }, []);

  const startWindowDrag = useCallback((event: MouseEvent<HTMLElement>) => {
    if (event.button !== 0) return;
    const target = event.target as HTMLElement;
    if (target.closest("button, input, textarea, a, .no-drag")) return;
    getCurrentWindow().startDragging().catch(console.error);
  }, []);

  const savePrefs = async () => {
    if (!config) return;
    await invoke("save_widget_config_cmd", { config });
    const aliases: Record<string, string> = {};
    if (claudeAlias.trim()) {
      aliases.default = claudeAlias.trim();
    }
    await invoke("save_accounts_config_cmd", {
      config: { claude: { auto_discover: true, aliases, hidden: [] } },
    });
    await invoke("set_always_on_top", { alwaysOnTop: config.always_on_top });
    await invoke("set_launch_at_login", { enabled: config.launch_at_login });
    setShowPrefs(false);
    refresh();
  };

  return (
    <div
      className="flex h-screen cursor-grab flex-col overflow-hidden rounded-2xl border border-white/10 bg-black/40 shadow-2xl backdrop-blur-xl active:cursor-grabbing"
      onMouseDown={startWindowDrag}
    >
      <header className="flex items-center justify-between border-b border-white/10 px-3 py-2">
        <div className="text-[12px] font-semibold text-white/90">AI Usage</div>
        <div className="no-drag flex gap-1">
          <button
            type="button"
            onClick={refresh}
            className="rounded px-2 py-0.5 text-[10px] text-white/60 hover:bg-white/10 hover:text-white"
            title="Refresh now"
          >
            ↻
          </button>
          <button
            type="button"
            onClick={() => invoke("hide_panel")}
            className="rounded px-2 py-0.5 text-[10px] text-white/60 hover:bg-white/10 hover:text-white"
            title="Hide to menu bar"
          >
            ✕
          </button>
          <button
            type="button"
            onClick={() => setShowPrefs((v) => !v)}
            className="rounded px-2 py-0.5 text-[10px] text-white/60 hover:bg-white/10 hover:text-white"
          >
            ⚙
          </button>
        </div>
      </header>

      {showPrefs ? (
        <div className="no-drag flex-1 space-y-3 overflow-y-auto p-3 text-white">
          <label className="block text-[11px] text-white/70">
            Poll interval (seconds)
            <input
              type="number"
              min={60}
              max={3600}
              value={config?.poll_interval_seconds ?? 600}
              onChange={(e) =>
                setConfig((c) =>
                  c ? { ...c, poll_interval_seconds: Number(e.target.value) } : c,
                )
              }
              className="mt-1 w-full rounded border border-white/10 bg-black/40 px-2 py-1 text-[12px]"
            />
          </label>
          <label className="flex items-center gap-2 text-[11px] text-white/70">
            <input
              type="checkbox"
              checked={config?.show_account_info ?? true}
              onChange={(e) =>
                setConfig((c) =>
                  c ? { ...c, show_account_info: e.target.checked } : c,
                )
              }
            />
            Show account email
          </label>
          <div className="space-y-2 text-[11px] text-white/70">
            <div>Providers</div>
            {(
              [
                ["enable_claude", "Claude"],
                ["enable_codex", "Codex"],
                ["enable_cursor", "Cursor"],
                ["enable_antigravity", "Antigravity"],
              ] as const
            ).map(([key, label]) => (
              <label key={key} className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={config?.[key] ?? (key === "enable_claude" || key === "enable_codex")}
                  onChange={(e) =>
                    setConfig((c) => (c ? { ...c, [key]: e.target.checked } : c))
                  }
                />
                {label}
              </label>
            ))}
          </div>
          <label className="flex items-center gap-2 text-[11px] text-white/70">
            <input
              type="checkbox"
              checked={config?.launch_at_login ?? false}
              onChange={(e) =>
                setConfig((c) => (c ? { ...c, launch_at_login: e.target.checked } : c))
              }
            />
            Launch at login
          </label>
          <label className="flex items-center gap-2 text-[11px] text-white/70">
            <input
              type="checkbox"
              checked={config?.always_on_top ?? true}
              onChange={(e) =>
                setConfig((c) => (c ? { ...c, always_on_top: e.target.checked } : c))
              }
            />
            Always on top
          </label>
          <label className="block text-[11px] text-white/70">
            Claude display name (optional)
            <input
              type="text"
              value={claudeAlias}
              onChange={(e) => setClaudeAlias(e.target.value)}
              placeholder="Claude"
              className="mt-1 w-full rounded border border-white/10 bg-black/40 px-2 py-1 text-[12px]"
            />
          </label>
          <button
            type="button"
            onClick={savePrefs}
            className="w-full rounded bg-white/15 py-1.5 text-[11px] text-white hover:bg-white/25"
          >
            Save
          </button>
        </div>
      ) : (
        <main className="flex-1 space-y-2 overflow-y-auto p-2">
          {loading && !snapshot && (
            <div className="p-4 text-center text-[11px] text-white/50">Loading…</div>
          )}
          {snapshot?.accounts.map((account) => (
            <AccountCard
              key={account.account_id}
              account={account}
              showAccountEmail={config?.show_account_info ?? true}
            />
          ))}
          {snapshot && (
            <div className="px-1 py-1 text-center text-[9px] text-white/30">
              Updated {new Date(snapshot.refreshed_at).toLocaleTimeString()}
            </div>
          )}
        </main>
      )}
    </div>
  );
}

export default App;
