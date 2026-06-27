use crate::keychain::{read_antigravity_credential, AntigravityCredential};
use crate::types::{AccountStatus, UsageWindow};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;

/// OAuth client credentials are injected at **build time** via `ANTIGRAVITY_OAUTH_*` env vars
/// (see `.env.example`). These are the public Antigravity CLI (`agy`) desktop client values.
const OAUTH_CLIENT_ID: &str = env!("ANTIGRAVITY_OAUTH_CLIENT_ID");
const OAUTH_CLIENT_SECRET: &str = env!("ANTIGRAVITY_OAUTH_CLIENT_SECRET");

const DAILY_API: &str = "https://daily-cloudcode-pa.googleapis.com";
const USER_AGENT: &str = "antigravity/1.0.10 (Macintosh; Intel Mac OS X 10_15_7)";

#[derive(Debug, Deserialize)]
struct LoadCodeAssistResponse {
    #[serde(rename = "cloudaicompanionProject")]
    cloud_ai_companion_project: Option<String>,
    #[serde(rename = "currentTier")]
    current_tier: Option<TierInfo>,
    #[serde(rename = "paidTier")]
    paid_tier: Option<TierInfo>,
}

#[derive(Debug, Deserialize)]
struct TierInfo {
    name: Option<String>,
    id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QuotaSummaryResponse {
    groups: Vec<QuotaSummaryGroup>,
}

#[derive(Debug, Deserialize)]
struct QuotaSummaryGroup {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    buckets: Vec<QuotaSummaryBucket>,
}

#[derive(Debug, Deserialize)]
struct QuotaSummaryBucket {
    window: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RefreshTokenResponse {
    access_token: String,
    #[serde(default)]
    expires_in: Option<i64>,
}

struct ApiError {
    status: u16,
    message: String,
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())
}

fn tier_name(tier: &TierInfo) -> Option<String> {
    tier.name.clone().or_else(|| tier.id.clone())
}

fn token_needs_refresh(cred: &AntigravityCredential) -> bool {
    let Some(expires_at) = &cred.expires_at else {
        return false;
    };
    let Ok(expiry) = DateTime::parse_from_rfc3339(expires_at) else {
        return false;
    };
    expiry.with_timezone(&Utc) <= Utc::now() + chrono::Duration::minutes(2)
}

async fn refresh_access_token(refresh_token: &str) -> Result<String, String> {
    if OAUTH_CLIENT_ID == "unset" || OAUTH_CLIENT_SECRET == "unset" {
        return Err(
            "Antigravity OAuth not configured — copy .env.example to .env and rebuild".into(),
        );
    }

    let client = http_client()?;
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .header("User-Agent", USER_AGENT)
        .form(&[
            ("client_id", OAUTH_CLIENT_ID),
            ("client_secret", OAUTH_CLIENT_SECRET),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "OAuth refresh failed (HTTP {}) — open `agy` to sign in again",
            response.status()
        ));
    }

    let body: RefreshTokenResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(body.access_token)
}

async fn ensure_access_token(mut cred: AntigravityCredential) -> Result<String, String> {
    if !token_needs_refresh(&cred) {
        return Ok(cred.access_token);
    }

    let refresh = cred
        .refresh_token
        .as_deref()
        .ok_or_else(|| "Token expired — open `agy` to sign in again".to_string())?;

    cred.access_token = refresh_access_token(refresh).await?;
    Ok(cred.access_token)
}

fn used_percent_from_remaining(remaining_fraction: Option<f64>, reset_time: &Option<String>) -> f64 {
    match remaining_fraction {
        Some(remaining) => ((1.0 - remaining) * 100.0).clamp(0.0, 100.0),
        None if reset_time.is_some() => 100.0,
        None => 0.0,
    }
}

fn pick_weekly_bucket<'a>(buckets: &'a [QuotaSummaryBucket]) -> Option<&'a QuotaSummaryBucket> {
    buckets.iter().find(|b| {
        b.window
            .as_deref()
            .map(|w| w.to_lowercase().contains("week"))
            .unwrap_or(false)
            || b.display_name
                .as_deref()
                .map(|d| d.to_lowercase().contains("week"))
                .unwrap_or(false)
    })
}

fn normalize_plan_name(plan: Option<String>) -> Option<String> {
    plan.map(|p| {
        p.trim()
            .strip_prefix("Antigravity ")
            .or_else(|| p.strip_prefix("antigravity "))
            .map(str::to_string)
            .unwrap_or(p)
    })
}

fn normalize_group_label(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("gemini") {
        "Gemini models".to_string()
    } else if lower.contains("claude") || lower.contains("gpt") {
        "Claude & GPT models".to_string()
    } else {
        name.to_string()
    }
}

fn windows_from_quota_summary(summary: &QuotaSummaryResponse) -> Vec<UsageWindow> {
    let mut windows = Vec::new();

    for group in &summary.groups {
        let Some(bucket) = pick_weekly_bucket(&group.buckets) else {
            continue;
        };

        let label = group
            .display_name
            .as_deref()
            .map(normalize_group_label)
            .unwrap_or_else(|| "Weekly".to_string());

        let used_percent =
            used_percent_from_remaining(bucket.remaining_fraction, &bucket.reset_time);

        windows.push(UsageWindow {
            label,
            used_percent,
            resets_at: bucket.reset_time.clone(),
            remaining_label: None,
        });
    }

    windows
}

async fn post_json<T: for<'de> Deserialize<'de>>(
    client: &reqwest::Client,
    path: &str,
    access_token: &str,
    body: serde_json::Value,
) -> Result<T, ApiError> {
    let url = format!("{DAILY_API}{path}");
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .json(&body)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            message: e.to_string(),
        })?;

    let status = response.status().as_u16();
    if response.status().is_success() {
        return response.json().await.map_err(|e| ApiError {
            status,
            message: e.to_string(),
        });
    }

    Err(ApiError {
        status,
        message: format!("HTTP {status} from {url}"),
    })
}

async fn load_code_assist(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<LoadCodeAssistResponse, ApiError> {
    let body = json!({
        "metadata": {
            "ideType": "ANTIGRAVITY",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI"
        }
    });
    post_json(client, "/v1internal:loadCodeAssist", access_token, body).await
}

async fn fetch_quota_summary(
    client: &reqwest::Client,
    access_token: &str,
    project: Option<&str>,
) -> Result<QuotaSummaryResponse, ApiError> {
    let body = match project {
        Some(pid) => json!({ "project": pid }),
        None => json!({}),
    };
    post_json(
        client,
        "/v1internal:retrieveUserQuotaSummary",
        access_token,
        body,
    )
    .await
}

async fn fetch_google_email(client: &reqwest::Client, access_token: &str) -> Option<String> {
    let response = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let body: serde_json::Value = response.json().await.ok()?;
    body.get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

async fn fetch_windows_with_token(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<Vec<UsageWindow>, ApiError> {
    let project = match load_code_assist(client, access_token).await {
        Ok(data) => data.cloud_ai_companion_project,
        Err(e) => return Err(e),
    };

    if let Ok(summary) = fetch_quota_summary(client, access_token, project.as_deref()).await {
        let windows = windows_from_quota_summary(&summary);
        if !windows.is_empty() {
            return Ok(windows);
        }
    }

    if project.is_some() {
        if let Ok(summary) = fetch_quota_summary(client, access_token, None).await {
            let windows = windows_from_quota_summary(&summary);
            if !windows.is_empty() {
                return Ok(windows);
            }
        }
    }

    Err(ApiError {
        status: 0,
        message: "No quota groups in retrieveUserQuotaSummary".to_string(),
    })
}

async fn fetch_quota_with_auth(
    client: &reqwest::Client,
    cred: AntigravityCredential,
) -> Result<Vec<UsageWindow>, String> {
    let access_token = ensure_access_token(cred.clone()).await?;

    match fetch_windows_with_token(client, &access_token).await {
        Ok(windows) => Ok(windows),
        Err(api_err) if api_err.status == 401 || api_err.status == 403 => {
            let refresh = cred.refresh_token.as_deref().ok_or_else(|| {
                "Session expired — open `agy` and run `/usage` once".to_string()
            })?;
            let refreshed = refresh_access_token(refresh).await?;
            fetch_windows_with_token(client, &refreshed)
                .await
                .map_err(|e| e.message)
        }
        Err(api_err) => Err(api_err.message),
    }
}

fn antigravity_error(message: String) -> AccountStatus {
    AccountStatus {
        account_id: "antigravity-default".to_string(),
        tool: "antigravity".to_string(),
        display_name: "Antigravity".to_string(),
        account_email: None,
        plan: None,
        ok: false,
        windows: vec![],
        banked_resets: None,
        updated_at: Utc::now().to_rfc3339(),
        error: Some(message),
    }
}

pub async fn fetch_antigravity_account() -> AccountStatus {
    let updated_at = Utc::now().to_rfc3339();

    let cred = match read_antigravity_credential() {
        Some(c) => c,
        None => {
            return antigravity_error(
                "Not signed in — run `agy` and complete Google login".to_string(),
            );
        }
    };

    let client = match http_client() {
        Ok(c) => c,
        Err(e) => return antigravity_error(e),
    };

    let cred_for_fetch = cred;
    let access_token = match ensure_access_token(cred_for_fetch.clone()).await {
        Ok(t) => t,
        Err(e) => return antigravity_error(e),
    };

    let account_email = fetch_google_email(&client, &access_token).await;

    let plan = match load_code_assist(&client, &access_token).await {
        Ok(data) => normalize_plan_name(
            data.paid_tier
                .as_ref()
                .and_then(tier_name)
                .or_else(|| data.current_tier.as_ref().and_then(tier_name)),
        ),
        Err(_) => None,
    };

    let windows = match fetch_quota_with_auth(&client, cred_for_fetch).await {
        Ok(w) if !w.is_empty() => w,
        Ok(_) => {
            return AccountStatus {
                account_id: "antigravity-default".to_string(),
                tool: "antigravity".to_string(),
                display_name: "Antigravity".to_string(),
                account_email,
                plan,
                ok: false,
                windows: vec![],
                banked_resets: None,
                updated_at,
                error: Some("No quota data returned — try `/usage` in `agy`".to_string()),
            };
        }
        Err(e) => {
            return AccountStatus {
                account_id: "antigravity-default".to_string(),
                tool: "antigravity".to_string(),
                display_name: "Antigravity".to_string(),
                account_email,
                plan,
                ok: false,
                windows: vec![],
                banked_resets: None,
                updated_at,
                error: Some(e),
            };
        }
    };

    AccountStatus {
        account_id: "antigravity-default".to_string(),
        tool: "antigravity".to_string(),
        display_name: "Antigravity".to_string(),
        account_email,
        plan,
        ok: true,
        windows,
        banked_resets: None,
        updated_at,
        error: None,
    }
}
