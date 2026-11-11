use std::{
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};
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

pub static RIOT_CLIENT_VERSION: OnceLock<String> = OnceLock::new();

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
