use crate::{
    anyhow, bail,
    error::Result,
    globals::{CLIENT, RIOT_CLIENT_PRIVATE_SETTINGS_PATH},
    riot::{oauth, settings},
};
use serde_json::{json, Value};
use std::path::Path;

/// Authenticated Riot API credentials required for Valorant Player Data endpoints.
#[derive(Clone, Debug)]
pub struct RiotAuth {
    pub access_token: String,
    pub entitlements: String,
    pub puuid: String,
    pub region: String,
}

/// Fetch `Name#Tag` from the currently active Riot Client session file.
pub fn fetch_game_id_from_session() -> Result<String> {
    let yaml_path = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;
    if !yaml_path.exists() {
        bail!("No active Riot session found");
    }

    let saved = settings::read_saved_tokens(yaml_path)?;
    settings::decode_game_id(&saved.id_token)
}

/// Exchange a saved Riot session file for API access token, entitlements JWT, PUUID, and region.
pub fn authenticate(yaml_path: &Path) -> Result<RiotAuth> {
    let saved = settings::read_saved_tokens(yaml_path)?;
    let (access_token, id_token) = oauth::exchange_tokens(&saved)?;
    let entitlements = fetch_entitlements(&access_token)?;
    let puuid = fetch_puuid(&access_token)?;
    let region = fetch_region(&access_token, &id_token)?;

    Ok(RiotAuth {
        access_token,
        entitlements,
        puuid,
        region,
    })
}

fn fetch_entitlements(access_token: &str) -> Result<String> {
    let auth = format!("Bearer {}", access_token);
    let mut resp = CLIENT
        .post("https://entitlements.auth.riotgames.com/api/token/v1")
        .header("Authorization", auth.as_str())
        .header("Content-Type", "application/json")
        .send_json(&json!({}))
        .map_err(|_| anyhow!("Failed to fetch entitlements token"))?;

    let json: Value = resp
        .body_mut()
        .read_json()
        .map_err(|_| anyhow!("Invalid entitlements response"))?;

    json["entitlements_token"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("No entitlements token in response"))
}

fn fetch_puuid(access_token: &str) -> Result<String> {
    let auth = format!("Bearer {}", access_token);
    let mut resp = CLIENT
        .post("https://auth.riotgames.com/userinfo")
        .header("Authorization", auth.as_str())
        .send_json(&json!({}))
        .map_err(|_| anyhow!("Failed to fetch player info"))?;

    let json: Value = resp
        .body_mut()
        .read_json()
        .map_err(|_| anyhow!("Invalid player info response"))?;

    json["sub"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("No account ID in player info"))
}

fn fetch_region(access_token: &str, id_token: &str) -> Result<String> {
    let auth = format!("Bearer {}", access_token);
    let mut resp = CLIENT
        .put("https://riot-geo.pas.si.riotgames.com/pas/v1/product/valorant")
        .header("Authorization", auth.as_str())
        .header("Content-Type", "application/json")
        .send_json(&json!({ "id_token": id_token }))
        .map_err(|_| anyhow!("Failed to fetch account region"))?;

    let json: Value = resp
        .body_mut()
        .read_json()
        .map_err(|_| anyhow!("Invalid region response"))?;

    let live = json["affinities"]["live"]
        .as_str()
        .ok_or_else(|| anyhow!("No region affinity in response"))?;

    Ok(match live {
        "latam" | "br" => "na",
        other => other,
    }
    .to_string())
}
