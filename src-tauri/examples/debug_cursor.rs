use rusqlite::Connection;
use serde_json::Value;
use std::path::PathBuf;

fn cursor_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library/Application Support/Cursor/User/globalStorage/state.vscdb")
}

fn read_cursor_value(key: &str) -> Option<String> {
    let conn = Connection::open(cursor_db_path()).ok()?;
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

fn redact_value(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, val) in map {
                if k.to_lowercase().contains("token") {
                    out.insert(k.clone(), Value::String("<redacted>".to_string()));
                } else {
                    out.insert(k.clone(), redact_value(val));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(redact_value).collect()),
        other => other.clone(),
    }
}

#[tokio::main]
async fn main() {
    let token = match read_cursor_value("cursorAuth/accessToken") {
        Some(t) if !t.is_empty() => t,
        _ => {
            eprintln!("no cursor token");
            return;
        }
    };

    let auth = reqwest::Client::new()
        .get("https://api2.cursor.sh/auth/usage")
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    println!("auth/usage:\n{}", serde_json::to_string_pretty(&redact_value(&auth)).unwrap());

    let plan = reqwest::Client::new()
        .post("https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("Connect-Protocol-Version", "1")
        .body("{}")
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    println!("\nplan usage:\n{}", serde_json::to_string_pretty(&redact_value(&plan)).unwrap());
}
