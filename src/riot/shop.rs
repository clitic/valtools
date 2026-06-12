use super::api;
use crate::accounts::ACCOUNTS_DIR;
use crate::error::{AppError, AppResult};
use std::path::PathBuf;

pub struct SkinOffer {
    pub name: String,
    pub cost: u32,
    pub icon_url: String,
    pub icon_path: Option<PathBuf>,
}

pub struct AccountStore {
    pub game_id: String,
    pub vp: u32,
    pub skins: Vec<SkinOffer>,
    pub reset_secs: u64,
}

static DOWNLOADS_DIR: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(|| {
    let dir = crate::LOCAL_APP_DATA.join("ValTools").join("Downloads");
    std::fs::create_dir_all(&dir).ok();
    dir
});

fn download_skin_image(url: &str, uuid: &str) -> Option<PathBuf> {
    let path = DOWNLOADS_DIR.join(format!("{}.png", uuid));

    if path.exists() {
        return Some(path);
    }

    let bytes = api::download_bytes(url)?;
    std::fs::write(&path, &bytes).ok()?;
    Some(path)
}

/// Download a single skin image by URL. Returns the path if successful.
pub fn download_single_image(url: &str) -> Option<PathBuf> {
    download_skin_image(url, icon_uuid(url))
}

fn get_skin_info(skin_uuid: &str) -> (String, String) {
    let url = format!(
        "https://valorant-api.com/v1/weapons/skinlevels/{}",
        skin_uuid
    );

    if let Ok(json) = api::get_json(&url, &[]) {
        let name = json["data"]["displayName"]
            .as_str()
            .unwrap_or("Unknown Skin")
            .to_string();
        let icon_url = json["data"]["displayIcon"]
            .as_str()
            .unwrap_or("")
            .to_string();
        return (name, icon_url);
    }

    ("Unknown Skin".to_string(), String::new())
}

/// Extract a stable UUID from icon URL.
/// URL format: https://media.valorant-api.com/weaponskinlevels/{uuid}/displayicon.png
fn icon_uuid(url: &str) -> &str {
    let mut parts = url.rsplit('/');
    parts.next(); // skip "displayicon.png"
    parts.next().unwrap_or("unknown")
}

/// Check if a skin image is already cached on disk.
fn cached_icon_path(icon_url: &str) -> Option<PathBuf> {
    if icon_url.is_empty() {
        return None;
    }
    let path = DOWNLOADS_DIR.join(format!("{}.png", icon_uuid(icon_url)));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn seconds_to_hms(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;

    if h > 0 {
        format!("{}h{}m", h, m)
    } else {
        format!("{}m", m)
    }
}

pub fn format_reset_time(secs: u64) -> String {
    seconds_to_hms(secs)
}

/// Fetch the full store for a given account label, sequentially.
/// Calls `on_progress` with a status string after each step.
pub fn fetch_account_store(
    label: &str,
    region: &str,
    on_progress: impl Fn(&str),
) -> AppResult<AccountStore> {
    let yaml_path = ACCOUNTS_DIR
        .join(label)
        .join("RiotGamesPrivateSettings.yaml");

    if !yaml_path.exists() {
        return Err(AppError::App(format!("No session file for '{}'", label)));
    }

    on_progress("Loading session...");
    let ssid = api::extract_ssid(&yaml_path)?;

    on_progress("Authenticating with Riot...");
    let access_token = api::get_access_token(&ssid)?;

    on_progress("Fetching entitlements...");
    let entitlements = api::get_entitlements_token(&access_token)?;

    on_progress("Fetching player info...");
    let (game_id, puuid) = api::get_player_info(&access_token)?;

    on_progress("Checking client version...");
    let client_version = api::cached_client_version()?;

    on_progress("Fetching storefront...");
    let store_data = api::get_storefront(
        &access_token,
        &entitlements,
        &puuid,
        &client_version,
        region,
    )?;

    on_progress("Fetching VP balance...");
    let vp = api::get_wallet(
        &access_token,
        &entitlements,
        &puuid,
        &client_version,
        region,
    )?;

    let offers = store_data["SkinsPanelLayout"]["SingleItemStoreOffers"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let reset_secs = store_data["SkinsPanelLayout"]["SingleItemOffersRemainingDurationInSeconds"]
        .as_u64()
        .unwrap_or(0);

    on_progress("Resolving skin details...");
    let mut skins = Vec::new();
    for (i, offer) in offers.iter().enumerate() {
        let uuid = offer["OfferID"].as_str().unwrap_or("").to_string();
        let cost = offer["Cost"]["85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741"]
            .as_u64()
            .unwrap_or(0) as u32;

        on_progress(&format!("Resolving skin {}/{}...", i + 1, offers.len()));
        let (name, icon_url) = get_skin_info(&uuid);
        let icon_path = cached_icon_path(&icon_url);
        skins.push(SkinOffer {
            name,
            cost,
            icon_url,
            icon_path,
        });
    }

    Ok(AccountStore {
        game_id,
        vp,
        skins,
        reset_secs,
    })
}

/// Fetch game_id (Name#Tag) from the current active Riot session.
pub fn fetch_game_id_from_session() -> AppResult<String> {
    let yaml_path = crate::LOCAL_APP_DATA
        .join("Riot Games")
        .join("Riot Client")
        .join("Data")
        .join("RiotGamesPrivateSettings.yaml");

    if !yaml_path.exists() {
        return Err(AppError::App("No active Riot session found".into()));
    }

    let ssid = api::extract_ssid(&yaml_path)?;
    let access_token = api::get_access_token(&ssid)?;
    let (game_id, _) = api::get_player_info(&access_token)?;
    Ok(game_id)
}
