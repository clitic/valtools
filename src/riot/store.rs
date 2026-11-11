use super::{auth, tiers::SkinTier};
use crate::globals::{CLIENT, RIOT_CLIENT_PRIVATE_SETTINGS_PATH, VALTOOLS_DOWNLOADS_DIR};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SkinOffer {
    pub name: String,
    pub cost: u32,
    pub icon_path: Option<PathBuf>,
    pub tier: SkinTier,
}

#[derive(Debug, Clone)]
pub struct AccountStore {
    pub vp: u32,
    pub rp: u32,
    pub skins: Vec<SkinOffer>,
    pub reset_secs: u64,
}

fn download_skin_image(url: &str, uuid: &str) -> Option<PathBuf> {
    if url.is_empty() {
        return None;
    }
    let path = VALTOOLS_DOWNLOADS_DIR.join(format!("{}.png", uuid));

    if path.exists() {
        return Some(path);
    }

    let mut resp = CLIENT.get(url).call().ok()?;
    let bytes = resp.body_mut().read_to_vec().ok()?;
    std::fs::write(&path, &bytes).ok()?;
    Some(path)
}

/// Download a single skin image by URL. Returns the path if successful.
pub fn download_single_image(url: &str) -> Option<PathBuf> {
    download_skin_image(url, icon_uuid(url))
}

pub fn get_skin_info(skin_uuid: &str) -> (String, String) {
    let url = format!(
        "https://valorant-api.com/v1/weapons/skinlevels/{}",
        skin_uuid
    );

    if let Ok(mut resp) = CLIENT.get(&url).call() {
        if let Ok(json) = resp.body_mut().read_json::<Value>() {
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

fn seconds_to_hms(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;

    if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}

pub fn format_reset_time(secs: u64) -> String {
    seconds_to_hms(secs)
}

/// Fetch game_id (Name#Tag) from the current active Riot session.
pub fn fetch_game_id_from_session() -> Result<String, String> {
    let yaml_path = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;

    if !yaml_path.exists() {
        return Err("No active Riot session found".into());
    }

    let tokens = auth::extract_session_tokens(yaml_path)?;
    let (game_id, _) = auth::get_player_info_from_id_token(&tokens.id_token)?;
    Ok(game_id)
}
