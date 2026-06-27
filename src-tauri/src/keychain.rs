use crate::types::ClaudeCredential;
use base64::{engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD}, Engine};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SERVICE_PREFIX: &str = "Claude Code-credentials";

#[derive(Debug, Deserialize)]
struct OAuthInner {
    #[serde(alias = "accessToken")]
    access_token: Option<String>,
    #[serde(alias = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(alias = "expiresAt")]
    expires_at: Option<i64>,
    #[serde(alias = "subscriptionType")]
    subscription_type: Option<String>,
}

fn dirs_fallback() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn parse_oauth_inner(value: &serde_json::Value) -> Option<OAuthInner> {
    if let Some(wrapped) = value.get("claudeAiOauth") {
        return serde_json::from_value(wrapped.clone()).ok();
    }
    serde_json::from_value(value.clone()).ok()
}

fn credential_from_json(
    service_name: &str,
    parsed: &serde_json::Value,
) -> Option<ClaudeCredential> {
    let inner = parse_oauth_inner(parsed)?;
    let access_token = inner.access_token?;
    Some(ClaudeCredential {
        account_key: "default".to_string(),
        service_name: service_name.to_string(),
        access_token,
        refresh_token: inner.refresh_token,
        expires_at: inner.expires_at,
        subscription_type: inner.subscription_type,
    })
}

fn read_credential_file(path: &Path, service_name: &str) -> Option<ClaudeCredential> {
    let content = fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    credential_from_json(service_name, &parsed)
}

fn read_credential_with_account(
    service_name: &str,
    account: Option<&str>,
) -> Option<ClaudeCredential> {
    let output = if let Some(a) = account {
        Command::new("/usr/bin/security")
            .args(["find-generic-password", "-s", service_name, "-a", a, "-w"])
            .output()
            .ok()?
    } else {
        Command::new("/usr/bin/security")
            .args(["find-generic-password", "-s", service_name, "-w"])
            .output()
            .ok()?
    };

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if json_str.is_empty() {
        return None;
    }

    let parsed: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    credential_from_json(service_name, &parsed)
}

/// Read the default Claude Code credential from Keychain (`Claude Code-credentials` + login user),
/// falling back to `~/.claude/.credentials.json`.
pub fn read_default_claude_credential() -> Option<ClaudeCredential> {
    let user = std::env::var("USER").ok();

    if let Some(cred) = read_credential_with_account(SERVICE_PREFIX, user.as_deref()) {
        return Some(cred);
    }

    let default_file = dirs_fallback().join(".claude").join(".credentials.json");
    read_credential_file(&default_file, SERVICE_PREFIX)
}

pub fn decode_jwt_email(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;

    for key in [
        "email",
        "preferred_username",
        "https://api.anthropic.com/email",
    ] {
        if let Some(email) = json.get(key).and_then(|v| v.as_str()) {
            if email.contains('@') {
                return Some(email.to_string());
            }
        }
    }

    json.get("https://api.anthropic.com/user")
        .and_then(|u| u.get("email"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn decode_jwt_nested_claim(token: &str, path: &[&str]) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let mut current = serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
    for key in path {
        current = current.get(*key)?.clone();
    }
    current.as_str().map(|s| s.to_string())
}

#[derive(Debug, Clone)]
pub struct AntigravityCredential {
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// RFC3339 expiry from keychain, when present.
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AntigravityKeyringPayload {
    token: AntigravityTokenBlob,
}

#[derive(Debug, Deserialize)]
struct AntigravityTokenBlob {
    access_token: String,
    refresh_token: Option<String>,
    expiry: Option<String>,
}

/// Read Antigravity CLI (`agy`) OAuth token from Keychain (`gemini` / `antigravity`).
pub fn read_antigravity_credential() -> Option<AntigravityCredential> {
    let output = Command::new("/usr/bin/security")
        .args([
            "find-generic-password",
            "-s",
            "gemini",
            "-a",
            "antigravity",
            "-w",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        return None;
    }

    let json_str = if let Some(b64) = raw.strip_prefix("go-keyring-base64:") {
        let bytes = STANDARD.decode(b64).ok()?;
        String::from_utf8(bytes).ok()?
    } else {
        raw
    };

    let parsed: AntigravityKeyringPayload = serde_json::from_str(&json_str).ok()?;
    Some(AntigravityCredential {
        access_token: parsed.token.access_token,
        refresh_token: parsed.token.refresh_token,
        expires_at: parsed.token.expiry,
    })
}
