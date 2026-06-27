use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn codex_home() -> PathBuf {
    std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".codex"))
                .unwrap_or_else(|_| PathBuf::from(".codex"))
        })
}

fn redact(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut out = serde_json::Map::new();
            for (k, val) in m {
                if k.contains("token") || k.contains("secret") {
                    out.insert(k.clone(), Value::String("<redacted>".into()));
                } else {
                    out.insert(k.clone(), redact(val));
                }
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(redact).collect()),
        other => other.clone(),
    }
}

#[tokio::main]
async fn main() {
    let auth_path = codex_home().join("auth.json");
    let auth: Value = serde_json::from_str(&fs::read_to_string(auth_path).unwrap()).unwrap();
    let token = auth["tokens"]["access_token"].as_str().unwrap();

    let endpoints = [
        "https://chatgpt.com/backend-api/codex/usage",
        "https://chatgpt.com/backend-api/wham/usage",
        "https://chatgpt.com/backend-api/codex/rate_limits",
        "https://chatgpt.com/backend-api/accounts/rate_limits",
        "https://chatgpt.com/backend-api/accounts/check",
    ];

    let client = reqwest::Client::new();
    for ep in endpoints {
        let resp = client
            .get(ep)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let preview = if body.len() > 500 {
            format!("{}...", &body[..500])
        } else {
            body.clone()
        };
        println!("{ep} -> {status}");
        if status.is_success() {
            if let Ok(json) = serde_json::from_str::<Value>(&body) {
                println!("{}", serde_json::to_string_pretty(&redact(&json)).unwrap());
            } else {
                println!("{preview}");
            }
        } else {
            println!("{preview}");
        }
        println!();
    }
}
