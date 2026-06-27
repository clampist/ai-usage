export interface UsageWindow {
  label: string;
  used_percent: number;
  resets_at: string | null;
  remaining_label?: string;
}

export interface AccountStatus {
  account_id: string;
  tool: "claude" | "codex" | "cursor" | "antigravity";
  display_name: string;
  account_email?: string;
  plan?: string;
  ok: boolean;
  windows: UsageWindow[];
  banked_resets?: number;
  updated_at: string;
  error?: string;
}

export interface WidgetSnapshot {
  accounts: AccountStatus[];
  refreshed_at: string;
}

export interface WidgetConfig {
  poll_interval_seconds: number;
  show_account_info: boolean;
  enable_claude: boolean;
  enable_codex: boolean;
  enable_cursor: boolean;
  enable_antigravity: boolean;
  cursor_included_model_key: string;
  always_on_top: boolean;
  window_x?: number | null;
  window_y?: number | null;
  window_width?: number | null;
  window_height?: number | null;
  launch_at_login: boolean;
}

export interface ClaudeAccountConfig {
  auto_discover: boolean;
  aliases: Record<string, string>;
  hidden: string[];
}

export interface AccountsConfig {
  claude: ClaudeAccountConfig;
}
