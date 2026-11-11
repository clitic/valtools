use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{path::PathBuf, sync::LazyLock};
use ureq::Agent;

pub static CLIENT: LazyLock<Agent> = LazyLock::new(|| Agent::new_with_defaults());

pub static LOCAL_APP_DATA: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(
        std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".into()),
    )
});

pub static RIOT_CLIENT_PRIVATE_SETTINGS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    LOCAL_APP_DATA
        .join("Riot Games")
        .join("Riot Client")
        .join("Data")
        .join("RiotGamesPrivateSettings.yaml")
});

pub static RIOT_CLIENT_PLATFORM: LazyLock<String> = LazyLock::new(|| {
    STANDARD.encode(
        serde_json::json!({
            "platformType": "PC",
            "platformOS": "Windows",
            "platformOSVersion": "10.0.19042.1.256.64bit",
            "platformChipset": "Unknown"
        })
        .to_string(),
    )
});

pub static RIOT_CLIENT_VERSION: LazyLock<String> = LazyLock::new(|| {
    if let Ok(mut resp) = CLIENT.get("https://valorant-api.com/v1/version").call() {
        if let Ok(json) = resp.body_mut().read_json::<Value>() {
            if let Some(version) = json["data"]["riotClientVersion"].as_str() {
                return version.to_owned();
            }
        }
    }

    String::new()
});

pub static VALORANT_LOCAL_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| LOCAL_APP_DATA.join("VALORANT"));

pub static VALTOOLS_CONFIG_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| LOCAL_APP_DATA.join("ValTools").join("active.txt"));

pub static VALTOOLS_ACCOUNTS_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| LOCAL_APP_DATA.join("ValTools").join("Accounts"));

pub static VALTOOLS_DOWNLOADS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = LOCAL_APP_DATA.join("ValTools").join("Downloads");
    std::fs::create_dir_all(&dir).ok();
    dir
});
