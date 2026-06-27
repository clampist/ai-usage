use crate::types::{AccountStatus, UsageWindow};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde_json::Value;
use std::path::PathBuf;

fn cursor_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Cursor")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb")
}

fn read_cursor_value(key: &str) -> Option<String> {
    let path = cursor_db_path();
    let conn = Connection::open(path).ok()?;
    let raw: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .ok()?;

    if let Ok(parsed) = serde_json::from_str::<String>(&raw) {
        return Some(parsed);
    }
    Some(raw.trim_matches('"').to_string())
}

fn parse_reset_timestamp(value: &Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        if let Ok(ms) = s.parse::<i64>() {
            if ms > 1_000_000_000_000 {
                return DateTime::from_timestamp_millis(ms).map(|dt| dt.to_rfc3339());
            }
        }
        return Some(s.to_string());
    }
    if let Some(ms) = value.as_i64() {
        if ms > 1_000_000_000_000 {
            return DateTime::from_timestamp_millis(ms).map(|dt| dt.to_rfc3339());
        }
        return DateTime::from_timestamp(ms, 0).map(|dt| dt.to_rfc3339());
    }
    None
}

fn parse_plan_usage_windows(body: &Value) -> Option<Vec<UsageWindow>> {
    let usage = body.get("planUsage").or_else(|| body.get("plan_usage"))?;

    let limit = usage
        .get("limit")
        .and_then(|v| v.as_i64())
        .filter(|&l| l > 0);
    let remaining = usage.get("remaining").and_then(|v| v.as_i64());
    let total_spend = usage
        .get("totalSpend")
        .or_else(|| usage.get("total_spend"))
        .and_then(|v| v.as_i64());

    let used_percent = match (total_spend, limit) {
        (Some(spend), Some(lim)) => (spend as f64 / lim as f64 * 100.0).clamp(0.0, 100.0),
        _ => usage
            .get("totalPercentUsed")
            .or_else(|| usage.get("total_percent_used"))
            .or_else(|| usage.get("apiPercentUsed"))
            .or_else(|| usage.get("api_percent_used"))
            .and_then(|v| v.as_f64())?
            .clamp(0.0, 100.0),
    };

    let remaining_label = match (remaining, limit, total_spend) {
        (Some(_rem), Some(lim), Some(spend)) => Some(format!(
            "${:.2} / ${:.2} used",
            spend as f64 / 100.0,
            lim as f64 / 100.0
        )),
        (Some(rem), _, _) => Some(format!("${:.2} remaining", rem as f64 / 100.0)),
        _ => body
            .get("displayMessage")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };

    let resets_at = body
        .get("billingCycleEnd")
        .or_else(|| body.get("billing_cycle_end"))
        .and_then(parse_reset_timestamp);

    Some(vec![UsageWindow {
        label: "Billing period".to_string(),
        used_percent,
        resets_at,
        remaining_label,
    }])
}

async fn fetch_plan_usage(access_token: &str) -> Result<Vec<UsageWindow>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .post("https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Content-Type", "application/json")
        .header("Connect-Protocol-Version", "1")
        .body("{}")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("plan usage HTTP {}", response.status()));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    if let Some(windows) = parse_plan_usage_windows(&body) {
        return Ok(windows);
    }

    Err(format!(
        "plan usage response missing fields; keys: {:?}",
        body.as_object().map(|o| o.keys().collect::<Vec<_>>())
    ))
}

fn parse_auth_usage(value: &Value, model_key: &str) -> Option<Vec<UsageWindow>> {
    let gpt4 = value
        .get(model_key)
        .or_else(|| value.get("gpt-4"))
        .or_else(|| {
            value.as_object().and_then(|obj| {
                obj.iter()
                    .find(|(k, v)| {
                        k.contains("gpt")
                            && (v.get("maxRequestUsage").is_some()
                                || v.get("max_request_usage").is_some())
                    })
                    .map(|(_, v)| v)
            })
        })?;

    let num_requests = gpt4
        .get("numRequests")
        .or_else(|| gpt4.get("num_requests"))
        .and_then(|v| v.as_i64())?;
    let max_requests = gpt4
        .get("maxRequestUsage")
        .or_else(|| gpt4.get("max_request_usage"))
        .and_then(|v| v.as_i64())
        .filter(|&m| m > 0)?;

    let used_percent =
        (num_requests as f64 / max_requests as f64 * 100.0).clamp(0.0, 100.0);
    let remaining = max_requests - num_requests;

    let resets_at = value
        .get("startOfMonth")
        .or_else(|| value.get("start_of_month"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(vec![UsageWindow {
        label: "Monthly requests".to_string(),
        used_percent,
        resets_at,
        remaining_label: Some(format!("{remaining} requests left")),
    }])
}

async fn fetch_auth_usage(access_token: &str, model_key: &str) -> Result<Vec<UsageWindow>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://api2.cursor.sh/auth/usage")
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("auth usage HTTP {}", response.status()));
    }

    let body: Value = response.json().await.map_err(|e| e.to_string())?;
    parse_auth_usage(&body, model_key).ok_or_else(|| {
        format!(
            "auth usage missing model bucket; keys: {:?}",
            body.as_object().map(|o| o.keys().collect::<Vec<_>>())
        )
    })
}

async fn fetch_usage_summary(access_token: &str) -> Result<Vec<UsageWindow>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://api2.cursor.sh/api/usage/summary")
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("usage summary HTTP {}", response.status()));
    }

    let body: Value = response.json().await.map_err(|e| e.to_string())?;

    if let Some(percent) = body
        .get("usagePercent")
        .or_else(|| body.get("usage_percent"))
        .and_then(|v| v.as_f64())
    {
        let resets_at = body
            .get("billingCycleEnd")
            .or_else(|| body.get("billing_cycle_end"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        return Ok(vec![UsageWindow {
            label: "Billing period".to_string(),
            used_percent: percent.clamp(0.0, 100.0),
            resets_at,
            remaining_label: None,
        }]);
    }

    Err("usage summary missing fields".to_string())
}

fn cursor_account(
    account_email: Option<String>,
    plan: Option<String>,
    ok: bool,
    windows: Vec<UsageWindow>,
    error: Option<String>,
) -> AccountStatus {
    AccountStatus {
        account_id: "cursor-default".to_string(),
        tool: "cursor".to_string(),
        display_name: "Cursor".to_string(),
        account_email,
        plan,
        ok,
        windows,
        banked_resets: None,
        updated_at: Utc::now().to_rfc3339(),
        error,
    }
}

pub async fn fetch_cursor_account(model_key: &str) -> AccountStatus {
    let access_token = match read_cursor_value("cursorAuth/accessToken") {
        Some(t) if !t.is_empty() => t,
        _ => {
            return cursor_account(
                None,
                None,
                false,
                vec![],
                Some("Not signed in to Cursor".to_string()),
            );
        }
    };

    let account_email = read_cursor_value("cursorAuth/cachedEmail");
    let plan = read_cursor_value("cursorAuth/stripeMembershipType");

    let mut errors = Vec::new();

    match fetch_plan_usage(&access_token).await {
        Ok(windows) if !windows.is_empty() => {
            return cursor_account(account_email, plan, true, windows, None);
        }
        Err(e) => errors.push(e),
        _ => {}
    }

    match fetch_auth_usage(&access_token, model_key).await {
        Ok(windows) if !windows.is_empty() => {
            return cursor_account(account_email, plan, true, windows, None);
        }
        Err(e) => errors.push(e),
        _ => {}
    }

    match fetch_usage_summary(&access_token).await {
        Ok(windows) if !windows.is_empty() => {
            return cursor_account(account_email, plan, true, windows, None);
        }
        Err(e) => errors.push(e),
        _ => {}
    }

    cursor_account(
        account_email,
        plan,
        false,
        vec![],
        Some(if errors.is_empty() {
            "Could not fetch Cursor usage".to_string()
        } else {
            errors.join("; ")
        }),
    )
}
