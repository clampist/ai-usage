mod antigravity;
mod claude;
mod codex;
mod cursor;

pub use antigravity::fetch_antigravity_account;
pub use claude::fetch_claude_account;
pub use codex::fetch_codex_account;
pub use cursor::fetch_cursor_account;

use crate::config::{load_accounts_config, load_widget_config};
use crate::types::{AccountStatus, WidgetSnapshot};
use chrono::Utc;

pub async fn fetch_all_usage() -> WidgetSnapshot {
    let accounts_config = load_accounts_config();
    let widget_config = load_widget_config();

    let mut accounts: Vec<AccountStatus> = Vec::new();

    if widget_config.enable_claude {
        accounts.push(
            fetch_claude_account(&accounts_config.claude.aliases).await,
        );
    }

    if widget_config.enable_codex {
        accounts.push(fetch_codex_account().await);
    }

    if widget_config.enable_cursor {
        accounts.push(
            fetch_cursor_account(&widget_config.cursor_included_model_key).await,
        );
    }

    if widget_config.enable_antigravity {
        accounts.push(fetch_antigravity_account().await);
    }

    WidgetSnapshot {
        accounts,
        refreshed_at: Utc::now().to_rfc3339(),
    }
}
