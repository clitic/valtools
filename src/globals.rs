use std::{path::PathBuf, sync::LazyLock};
use ureq::Agent;

pub static CLIENT: LazyLock<Agent> = LazyLock::new(Agent::new_with_defaults);

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

pub static VALTOOLS_DIR: LazyLock<PathBuf> = LazyLock::new(|| LOCAL_APP_DATA.join("ValTools"));
