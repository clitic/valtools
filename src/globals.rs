use std::{
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

/// Root %LOCALAPPDATA% directory for the current user.
pub static LOCAL_APP_DATA: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(
        std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".into()),
    )
});

/// Path to Riot Client private settings YAML.
pub static PRIVATE_SETTINGS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    LOCAL_APP_DATA
        .join("Riot Games")
        .join("Riot Client")
        .join("Data")
        .join("RiotGamesPrivateSettings.yaml")
});

/// Path to ValTools active account configuration file.
pub static CONFIG_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| LOCAL_APP_DATA.join("ValTools").join("active.txt"));

/// Directory storing backed-up Riot account sessions.
pub static ACCOUNTS_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| LOCAL_APP_DATA.join("ValTools").join("Accounts"));

/// VALORANT local app data directory (used for cache cleanup).
pub static VALORANT_LOCAL_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| LOCAL_APP_DATA.join("VALORANT"));

/// Directory storing cached skin downloads.
pub static DOWNLOADS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = LOCAL_APP_DATA.join("ValTools").join("Downloads");
    std::fs::create_dir_all(&dir).ok();
    dir
});

/// Shared HTTP agent configured with Riot client user agent.
pub static HTTP_AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    let config = ureq::config::Config::builder()
        .user_agent("RiotClient/99.0.0.0000000.0000000 rso-auth (Windows;10;;Professional, x64)")
        .build();
    ureq::Agent::new_with_config(config)
});

/// Cached Valorant client version string.
pub static CLIENT_VERSION: OnceLock<String> = OnceLock::new();

