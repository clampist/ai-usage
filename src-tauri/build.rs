use std::path::Path;

fn main() {
    let dotenv = load_dotenv_files();
    emit_antigravity_oauth_env(dotenv.as_deref());
    tauri_build::build();
}

fn load_dotenv_files() -> Option<std::path::PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let manifest = Path::new(&manifest_dir);
    let candidates = [manifest.join(".env"), manifest.join("../.env")];

    for path in candidates {
        if path.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
            return Some(path);
        }
    }
    None
}

fn read_dotenv_value(path: &Path, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (k, value) = line.split_once('=')?;
        if k.trim() != key {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn emit_antigravity_oauth_env(dotenv: Option<&Path>) {
    let client_id = std::env::var("ANTIGRAVITY_OAUTH_CLIENT_ID")
        .ok()
        .or_else(|| dotenv.and_then(|p| read_dotenv_value(p, "ANTIGRAVITY_OAUTH_CLIENT_ID")))
        .unwrap_or_else(|| "unset".into());
    let client_secret = std::env::var("ANTIGRAVITY_OAUTH_CLIENT_SECRET")
        .ok()
        .or_else(|| dotenv.and_then(|p| read_dotenv_value(p, "ANTIGRAVITY_OAUTH_CLIENT_SECRET")))
        .unwrap_or_else(|| "unset".into());

    println!("cargo:rustc-env=ANTIGRAVITY_OAUTH_CLIENT_ID={client_id}");
    println!("cargo:rustc-env=ANTIGRAVITY_OAUTH_CLIENT_SECRET={client_secret}");
    println!("cargo:rerun-if-env-changed=ANTIGRAVITY_OAUTH_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=ANTIGRAVITY_OAUTH_CLIENT_SECRET");
}
