use crate::{
    globals::CLIENT,
    riot::{
        models::{AccessoryOffer, AccessoryType},
        skins,
    },
};
use serde_json::Value;

/// Look up accessory metadata on `valorant-api.com`, cache its icon, and build an `AccessoryOffer`.
pub fn resolve_accessory_offer(item_type_id: &str, item_id: &str, cost: u32) -> AccessoryOffer {
    let initial_type = AccessoryType::from_item_type_id(item_type_id);
    let (name, icon_url, resolved_type) = fetch_accessory_metadata(initial_type, item_id);
    let icon_path = skins::download_and_cache_icon(&icon_url);

    AccessoryOffer {
        name,
        cost,
        icon_path,
        item_type: resolved_type,
    }
}

fn fetch_accessory_metadata(
    item_type: AccessoryType,
    item_id: &str,
) -> (String, String, AccessoryType) {
    if item_id.is_empty() {
        return ("Unknown Accessory".to_string(), String::new(), item_type);
    }

    match item_type {
        AccessoryType::Buddy => {
            let level_url = format!("https://valorant-api.com/v1/buddies/levels/{}", item_id);
            if let Some((name, icon)) = query_display_data(&level_url) {
                if !name.is_empty() {
                    return (name, icon, AccessoryType::Buddy);
                }
            }
            let buddy_url = format!("https://valorant-api.com/v1/buddies/{}", item_id);
            if let Some((name, icon)) = query_display_data(&buddy_url) {
                return (name, icon, AccessoryType::Buddy);
            }
            ("Gun Buddy".to_string(), String::new(), AccessoryType::Buddy)
        }
        AccessoryType::Card => {
            let url = format!("https://valorant-api.com/v1/playercards/{}", item_id);
            if let Ok(mut resp) = CLIENT.get(&url).call() {
                if let Ok(json) = resp.body_mut().read_json::<Value>() {
                    let data = &json["data"];
                    let name = data["displayName"]
                        .as_str()
                        .unwrap_or("Player Card")
                        .to_string();
                    let icon_url = data["smallArt"]
                        .as_str()
                        .or_else(|| data["displayIcon"].as_str())
                        .or_else(|| data["largeArt"].as_str())
                        .unwrap_or("")
                        .to_string();
                    return (name, icon_url, AccessoryType::Card);
                }
            }
            (
                "Player Card".to_string(),
                String::new(),
                AccessoryType::Card,
            )
        }
        AccessoryType::Spray => {
            let url = format!("https://valorant-api.com/v1/sprays/{}", item_id);
            if let Ok(mut resp) = CLIENT.get(&url).call() {
                if let Ok(json) = resp.body_mut().read_json::<Value>() {
                    let data = &json["data"];
                    let name = data["displayName"].as_str().unwrap_or("Spray").to_string();
                    let icon_url = data["fullTransparentIcon"]
                        .as_str()
                        .or_else(|| data["displayIcon"].as_str())
                        .unwrap_or("")
                        .to_string();
                    return (name, icon_url, AccessoryType::Spray);
                }
            }
            ("Spray".to_string(), String::new(), AccessoryType::Spray)
        }
        AccessoryType::Title => {
            let url = format!("https://valorant-api.com/v1/playertitles/{}", item_id);
            if let Ok(mut resp) = CLIENT.get(&url).call() {
                if let Ok(json) = resp.body_mut().read_json::<Value>() {
                    let data = &json["data"];
                    let name = data["titleText"]
                        .as_str()
                        .or_else(|| data["displayName"].as_str())
                        .unwrap_or("Player Title")
                        .to_string();
                    return (name, String::new(), AccessoryType::Title);
                }
            }
            (
                "Player Title".to_string(),
                String::new(),
                AccessoryType::Title,
            )
        }
        AccessoryType::Unknown => {
            let buddy_url = format!("https://valorant-api.com/v1/buddies/levels/{}", item_id);
            if let Some((name, icon)) = query_display_data(&buddy_url) {
                if !name.is_empty() {
                    return (name, icon, AccessoryType::Buddy);
                }
            }
            let spray_url = format!("https://valorant-api.com/v1/sprays/{}", item_id);
            if let Some((name, icon)) = query_display_data(&spray_url) {
                if !name.is_empty() {
                    return (name, icon, AccessoryType::Spray);
                }
            }
            let card_url = format!("https://valorant-api.com/v1/playercards/{}", item_id);
            if let Some((name, icon)) = query_display_data(&card_url) {
                if !name.is_empty() {
                    return (name, icon, AccessoryType::Card);
                }
            }
            let title_url = format!("https://valorant-api.com/v1/playertitles/{}", item_id);
            if let Some((name, icon)) = query_display_data(&title_url) {
                if !name.is_empty() {
                    return (name, icon, AccessoryType::Title);
                }
            }
            (
                "Accessory".to_string(),
                String::new(),
                AccessoryType::Unknown,
            )
        }
    }
}

fn query_display_data(url: &str) -> Option<(String, String)> {
    let mut resp = CLIENT.get(url).call().ok()?;
    let json: Value = resp.body_mut().read_json().ok()?;
    let data = &json["data"];
    let name = data["displayName"]
        .as_str()
        .or_else(|| data["titleText"].as_str())
        .unwrap_or("")
        .to_string();
    let icon = data["displayIcon"]
        .as_str()
        .or_else(|| data["smallArt"].as_str())
        .or_else(|| data["fullTransparentIcon"].as_str())
        .unwrap_or("")
        .to_string();
    Some((name, icon))
}
