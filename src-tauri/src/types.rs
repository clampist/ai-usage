use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    pub resets_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatus {
    pub account_id: String,
    pub tool: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    pub ok: bool,
    pub windows: Vec<UsageWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banked_resets: Option<u32>,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSnapshot {
    pub accounts: Vec<AccountStatus>,
    pub refreshed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeAccountConfig {
    #[serde(default = "default_true", rename = "autoDiscover")]
    pub auto_discover: bool,
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub hidden: Vec<String>,
}

impl Default for ClaudeAccountConfig {
    fn default() -> Self {
        Self {
            auto_discover: true,
            aliases: std::collections::HashMap::new(),
            hidden: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountsConfig {
    #[serde(default)]
    pub claude: ClaudeAccountConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    #[serde(default = "default_poll_interval")]
    pub poll_interval_seconds: u64,
    #[serde(default = "default_true")]
    pub show_account_info: bool,
    #[serde(default = "default_true")]
    pub enable_claude: bool,
    #[serde(default = "default_true")]
    pub enable_codex: bool,
    #[serde(default = "default_false")]
    pub enable_cursor: bool,
    #[serde(default = "default_false")]
    pub enable_antigravity: bool,
    #[serde(default = "default_included_model")]
    pub cursor_included_model_key: String,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default)]
    pub window_x: Option<f64>,
    #[serde(default)]
    pub window_y: Option<f64>,
    #[serde(default)]
    pub window_width: Option<f64>,
    #[serde(default)]
    pub window_height: Option<f64>,
    #[serde(default)]
    pub launch_at_login: bool,
}

fn default_poll_interval() -> u64 {
    600
}

fn default_included_model() -> String {
    "gpt-4".to_string()
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            poll_interval_seconds: default_poll_interval(),
            show_account_info: true,
            enable_claude: true,
            enable_codex: true,
            enable_cursor: false,
            enable_antigravity: false,
            cursor_included_model_key: default_included_model(),
            always_on_top: true,
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            launch_at_login: false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ClaudeCredential {
    pub account_key: String,
    pub service_name: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub subscription_type: Option<String>,
}
