use crate::keychain::decode_jwt_email;
use crate::types::{AccountStatus, UsageWindow};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct CodexAuth {
    tokens: Option<CodexTokens>,
}

#[derive(Debug, Deserialize)]
struct CodexTokens {
    access_token: Option<String>,
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexGlobalState {
    #[serde(rename = "electron-persisted-atom-state")]
    electron_persisted_atom_state: Option<ElectronState>,
}

#[derive(Debug, Deserialize)]
struct ElectronState {
    #[serde(rename = "rate-limit-reset-home-announcement-dismissal-by-account-id")]
    rate_limit_resets: Option<std::collections::HashMap<String, ResetDismissal>>,
}

#[derive(Debug, Deserialize)]
struct ResetDismissal {
    #[serde(rename = "availableCount")]
    available_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CodexUsageResponse {
    plan_type: Option<String>,
    email: Option<String>,
    rate_limit: Option<CodexRateLimitBlock>,
    #[serde(rename = "rate_limit_reset_credits")]
    rate_limit_reset_credits: Option<ResetCredits>,
}

#[derive(Debug, Deserialize)]
struct ResetCredits {
    available_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CodexRateLimitBlock {
    primary_window: Option<CodexRateWindow>,
    secondary_window: Option<CodexRateWindow>,
}

#[derive(Debug, Deserialize)]
struct CodexRateWindow {
    used_percent: Option<i32>,
    reset_at: Option<i64>,
    limit_window_seconds: Option<i64>,
}

fn codex_home() -> PathBuf {
    std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".codex"))
                .unwrap_or_else(|_| PathBuf::from(".codex"))
        })
}

fn read_banked_resets() -> Option<u32> {
    let path = codex_home().join(".codex-global-state.json");
    let content = fs::read_to_string(path).ok()?;
    let state: CodexGlobalState = serde_json::from_str(&content).ok()?;
    let resets = state
        .electron_persisted_atom_state?
        .rate_limit_resets?;

    resets
        .values()
        .filter_map(|r| r.available_count)
        .max()
}

fn window_label(seconds: Option<i64>, default: &str) -> String {
    match seconds {
        Some(18000) => "5-hour".to_string(),
        Some(604800) => "Weekly".to_string(),
        Some(s) if s >= 86400 => format!("{}-day", s / 86400),
        Some(s) if s >= 3600 => format!("{}-hour", s / 3600),
        _ => default.to_string(),
    }
}

fn parse_window(window: &CodexRateWindow, default_label: &str) -> Option<UsageWindow> {
    let used_percent = window.used_percent? as f64;
    let resets_at = window
        .reset_at
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .map(|dt| dt.to_rfc3339());

    Some(UsageWindow {
        label: window_label(window.limit_window_seconds, default_label),
        used_percent: used_percent.clamp(0.0, 100.0),
        resets_at,
        remaining_label: None,
    })
}

fn parse_usage_response(body: &CodexUsageResponse) -> Vec<UsageWindow> {
    let mut windows = Vec::new();
    if let Some(rate_limit) = &body.rate_limit {
        if let Some(primary) = &rate_limit.primary_window {
            if let Some(w) = parse_window(primary, "5-hour") {
                windows.push(w);
            }
        }
        if let Some(secondary) = &rate_limit.secondary_window {
            if let Some(w) = parse_window(secondary, "Weekly") {
                windows.push(w);
            }
        }
    }
    windows
}

async fn fetch_codex_usage(access_token: &str) -> Option<CodexUsageResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let endpoints = [
        "https://chatgpt.com/backend-api/codex/usage",
        "https://chatgpt.com/backend-api/wham/usage",
    ];

    for endpoint in endpoints {
        let response = client
            .get(endpoint)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Content-Type", "application/json")
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            continue;
        }

        if let Ok(body) = response.json::<CodexUsageResponse>().await {
            if body.rate_limit.is_some() {
                return Some(body);
            }
        }
    }

    None
}

pub async fn fetch_codex_account() -> AccountStatus {
    let updated_at = Utc::now().to_rfc3339();
    let auth_path = codex_home().join("auth.json");

    let auth_content = match fs::read_to_string(&auth_path) {
        Ok(c) => c,
        Err(e) => {
            return AccountStatus {
                account_id: "codex-default".to_string(),
                tool: "codex".to_string(),
                display_name: "Codex".to_string(),
                account_email: None,
                plan: None,
                ok: false,
                windows: vec![],
                banked_resets: None,
                updated_at,
                error: Some(format!("Cannot read auth.json: {e}")),
            };
        }
    };

    let auth: CodexAuth = match serde_json::from_str(&auth_content) {
        Ok(a) => a,
        Err(e) => {
            return AccountStatus {
                account_id: "codex-default".to_string(),
                tool: "codex".to_string(),
                display_name: "Codex".to_string(),
                account_email: None,
                plan: None,
                ok: false,
                windows: vec![],
                banked_resets: None,
                updated_at,
                error: Some(format!("Invalid auth.json: {e}")),
            };
        }
    };

    let token = auth
        .tokens
        .as_ref()
        .and_then(|t| t.access_token.clone())
        .or_else(|| {
            auth.tokens
                .as_ref()
                .and_then(|t| t.id_token.clone())
        });

    let Some(access_token) = token else {
        return AccountStatus {
            account_id: "codex-default".to_string(),
            tool: "codex".to_string(),
            display_name: "Codex".to_string(),
            account_email: None,
            plan: None,
            ok: false,
            windows: vec![],
            banked_resets: read_banked_resets(),
            updated_at,
            error: Some("Not logged in".to_string()),
        };
    };

    let mut account_email = decode_jwt_email(&access_token);
    let plan = crate::keychain::decode_jwt_nested_claim(
        &access_token,
        &["https://api.openai.com/auth", "chatgpt_plan_type"],
    )
    .or_else(|| {
        auth.tokens.as_ref().and_then(|t| {
            t.id_token.as_ref().and_then(|id| {
                crate::keychain::decode_jwt_nested_claim(
                    id,
                    &["https://api.openai.com/auth", "chatgpt_plan_type"],
                )
            })
        })
    });

    let banked_resets = read_banked_resets();

    match fetch_codex_usage(&access_token).await {
        Some(body) => {
            let windows = parse_usage_response(&body);
            let api_banked = body
                .rate_limit_reset_credits
                .and_then(|c| c.available_count);
            let banked = api_banked.or(banked_resets);
            let plan = body.plan_type.or(plan);
            if account_email.is_none() {
                account_email = body.email;
            }

            AccountStatus {
                account_id: "codex-default".to_string(),
                tool: "codex".to_string(),
                display_name: "Codex".to_string(),
                account_email,
                plan,
                ok: !windows.is_empty(),
                windows,
                banked_resets: banked,
                updated_at,
                error: None,
            }
        }
        None => AccountStatus {
            account_id: "codex-default".to_string(),
            tool: "codex".to_string(),
            display_name: "Codex".to_string(),
            account_email,
            plan: plan.clone(),
            ok: false,
            windows: vec![],
            banked_resets,
            updated_at,
            error: Some(
                "Usage API unavailable — open Codex app to view limits".to_string(),
            ),
        },
    }
}
