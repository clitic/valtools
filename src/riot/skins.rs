use crate::{
    globals::{CLIENT, VALTOOLS_DOWNLOADS_DIR},
    riot::models::{SkinOffer, SkinTier},
};
use serde_json::Value;
use std::path::PathBuf;

/// Look up skin metadata on `valorant-api.com`, cache its icon, and build a `SkinOffer`.
pub fn resolve_skin_offer(skin_uuid: &str, cost: u32) -> SkinOffer {
    let (name, icon_url) = fetch_skin_metadata(skin_uuid);
    let icon_path = download_and_cache_icon(&icon_url);

    SkinOffer {
        name,
        cost,
        icon_path,
        tier: SkinTier::from_cost(cost),
    }
}

fn fetch_skin_metadata(skin_uuid: &str) -> (String, String) {
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

fn download_and_cache_icon(url: &str) -> Option<PathBuf> {
    if url.is_empty() {
        return None;
    }

    // URL format: https://media.valorant-api.com/weaponskinlevels/{uuid}/displayicon.png
    let uuid = url.rsplit('/').nth(1).unwrap_or("unknown");
    let path = VALTOOLS_DOWNLOADS_DIR.join(format!("{}.png", uuid));
    if path.exists() {
        return Some(path);
    }

    let mut resp = CLIENT.get(url).call().ok()?;
    let bytes = resp.body_mut().read_to_vec().ok()?;
    std::fs::write(&path, &bytes).ok()?;
    Some(path)
}

