use ai_usage_lib::providers;

#[tokio::main]
async fn main() {
    let snapshot = providers::fetch_all_usage().await;
    println!("{}", serde_json::to_string_pretty(&snapshot).unwrap());
}
