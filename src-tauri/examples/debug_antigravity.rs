use ai_usage_lib::providers;

#[tokio::main]
async fn main() {
    let account = providers::fetch_antigravity_account().await;
    println!("{}", serde_json::to_string_pretty(&account).unwrap());
}
