use crate::keychain::{decode_jwt_email, read_default_claude_credential};
use crate::types::{AccountStatus, UsageWindow};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_ACCOUNT_KEY: &str = "default";

#[derive(Debug, Deserialize)]
struct UsageApiResponse {
    five_hour: Option<UsageBucket>,
    seven_day: Option<UsageBucket>,
    seven_day_sonnet: Option<UsageBucket>,
    extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Deserialize)]
struct UsageBucket {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExtraUsage {
    utilization: Option<f64>,
    resets_at: Option<String>,
    spent_usd: Option<f64>,
    limit_usd: Option<f64>,
}

fn clamp_percent(value: Option<f64>) -> f64 {
    value.unwrap_or(0.0).clamp(0.0, 100.0)
}

fn parse_windows(response: &UsageApiResponse) -> Vec<UsageWindow> {
    let mut windows = Vec::new();

    if let Some(fh) = &response.five_hour {
        windows.push(UsageWindow {
            label: "5-hour".to_string(),
            used_percent: clamp_percent(fh.utilization),
            resets_at: fh.resets_at.clone(),
            remaining_label: None,
        });
    }

    if let Some(wk) = &response.seven_day {
        windows.push(UsageWindow {
            label: "Weekly".to_string(),
            used_percent: clamp_percent(wk.utilization),
            resets_at: wk.resets_at.clone(),
            remaining_label: None,
        });
    } else if let Some(sonnet) = &response.seven_day_sonnet {
        windows.push(UsageWindow {
            label: "Sonnet weekly".to_string(),
            used_percent: clamp_percent(sonnet.utilization),
            resets_at: sonnet.resets_at.clone(),
            remaining_label: None,
        });
    }

    if let Some(extra) = &response.extra_usage {
        if extra.utilization.is_some() || extra.spent_usd.is_some() {
            let used = clamp_percent(extra.utilization);
            let remaining = match (extra.spent_usd, extra.limit_usd) {
                (Some(spent), Some(limit)) if limit > 0.0 => {
                    Some(format!("${:.2} / ${:.2}", spent, limit))
                }
                _ => None,
            };
            windows.push(UsageWindow {
                label: "Extra usage".to_string(),
                used_percent: used,
                resets_at: extra.resets_at.clone(),
                remaining_label: remaining,
            });
        }
    }

    windows
}

async fn fetch_usage_api(access_token: &str) -> Result<UsageApiResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    response.json().await.map_err(|e| e.to_string())
}

fn claude_display_name(aliases: &HashMap<String, String>) -> String {
    aliases
        .get(DEFAULT_ACCOUNT_KEY)
        .cloned()
        .unwrap_or_else(|| "Claude".to_string())
}

pub async fn fetch_claude_account(aliases: &HashMap<String, String>) -> AccountStatus {
    let updated_at = Utc::now().to_rfc3339();
    let display_name = claude_display_name(aliases);

    let cred = match read_default_claude_credential() {
        Some(c) => c,
        None => {
            return AccountStatus {
                account_id: "claude-default".to_string(),
                tool: "claude".to_string(),
                display_name,
                account_email: None,
                plan: None,
                ok: false,
                windows: vec![],
                banked_resets: None,
                updated_at,
                error: Some(
                    "Cannot read default Keychain credential (Claude Code-credentials)".to_string(),
                ),
            };
        }
    };

    let account_email = decode_jwt_email(&cred.access_token)
        .or_else(|| std::env::var("USER").ok());
    let plan = cred.subscription_type.clone();

    match fetch_usage_api(&cred.access_token).await {
        Ok(api) => {
            let windows = parse_windows(&api);
            let ok = !windows.is_empty();
            AccountStatus {
                account_id: "claude-default".to_string(),
                tool: "claude".to_string(),
                display_name,
                account_email,
                plan,
                ok,
                windows,
                banked_resets: None,
                updated_at,
                error: if ok {
                    None
                } else {
                    Some("No usage data returned".to_string())
                },
            }
        }
        Err(err) => AccountStatus {
            account_id: "claude-default".to_string(),
            tool: "claude".to_string(),
            display_name,
            account_email,
            plan,
            ok: false,
            windows: vec![],
            banked_resets: None,
            updated_at,
            error: Some(err),
        },
    }
}
