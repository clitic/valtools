use crate::{anyhow, bail, error::Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::path::Path;

/// Saved Riot Client credentials extracted from `RiotGamesPrivateSettings.yaml`.
pub struct SavedTokens {
    pub id_token: String,
    pub refresh_token: String,
    pub tdid: Option<String>,
}

/// Read saved Riot Client tokens from `RiotGamesPrivateSettings.yaml`.
pub fn read_saved_tokens(yaml_path: &Path) -> Result<SavedTokens> {
    let content =
        std::fs::read_to_string(yaml_path).map_err(|_| anyhow!("Failed to read Riot session"))?;
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|_| anyhow!("Invalid Riot session file"))?;

    let riot_client = &yaml["psl"]["authorization"]["riot-client"];

    let id_token = riot_client["id_token"]
        .as_str()
        .ok_or_else(|| anyhow!("No login token found in session"))?
        .to_string();

    let refresh_token = riot_client["refresh_token"]
        .as_str()
        .ok_or_else(|| anyhow!("No refresh token found in session"))?
        .to_string();

    let tdid = yaml["rso-authenticator"]["tdid"]["value"]
        .as_str()
        .map(str::to_string);

    Ok(SavedTokens {
        id_token,
        refresh_token,
        tdid,
    })
}

/// Decode `Name#Tag` from a JWT `id_token`.
pub fn decode_game_id(id_token: &str) -> Result<String> {
    let payload = decode_jwt_payload(id_token)?;

    if payload["sub"].as_str().is_none() {
        bail!("No account ID found in login token");
    }

    let game_name = payload["acct"]["game_name"].as_str().unwrap_or("Unknown");
    let tag_line = payload["acct"]["tag_line"].as_str().unwrap_or("0000");

    Ok(format!("{}#{}", game_name, tag_line))
}

fn decode_jwt_payload(jwt: &str) -> Result<Value> {
    let payload_b64 = jwt
        .split('.')
        .nth(1)
        .ok_or_else(|| anyhow!("Invalid login token format"))?;

    let payload_bytes = STANDARD
        .decode(payload_b64)
        .or_else(|_| {
            let padded = match payload_b64.len() % 4 {
                2 => format!("{}==", payload_b64),
                3 => format!("{}=", payload_b64),
                _ => payload_b64.to_string(),
            };
            STANDARD.decode(&padded)
        })
        .map_err(|_| anyhow!("Failed to decode login token"))?;

    serde_json::from_slice(&payload_bytes).map_err(|_| anyhow!("Invalid login token payload"))
}

