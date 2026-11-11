use crate::globals::{CLIENT, RIOT_CLIENT_PRIVATE_SETTINGS_PATH};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug)]
pub struct RiotAuth {
    pub access_token: String,
    pub entitlements: String,
    pub puuid: String,
    pub region: String,
}

struct SessionTokens {
    id_token: String,
    refresh_token: String,
    tdid: Option<String>,
}

/// Fetch game_id (Name#Tag) from the current active Riot session.
pub fn fetch_game_id_from_session() -> Result<String, String> {
    let yaml_path = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;
    if !yaml_path.exists() {
        return Err("No active Riot session found".into());
    }

    let tokens = extract_session_tokens(yaml_path)?;
    decode_game_id_from_id_token(&tokens.id_token)
}

/// Exchange saved Riot session tokens for API access token, entitlements JWT, PUUID, and region.
pub fn authenticate(yaml_path: &Path) -> Result<RiotAuth, String> {
    let tokens = extract_session_tokens(yaml_path)?;
    let (access_token, id_token) = get_auth_tokens(&tokens)?;
    let entitlements = get_entitlements_token(&access_token)?;
    let puuid = get_player_puuid(&access_token)?;
    let region = get_region(&access_token, &id_token)?;

    Ok(RiotAuth {
        access_token,
        entitlements,
        puuid,
        region,
    })
}

fn extract_session_tokens(yaml_path: &Path) -> Result<SessionTokens, String> {
    let content = std::fs::read_to_string(yaml_path).map_err(|e| e.to_string())?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;

    let riot_client = &yaml["psl"]["authorization"]["riot-client"];

    let id_token = riot_client["id_token"]
        .as_str()
        .ok_or_else(|| "No id_token found in settings".to_string())?
        .to_string();

    let refresh_token = riot_client["refresh_token"]
        .as_str()
        .ok_or_else(|| "No refresh_token found in settings".to_string())?
        .to_string();

    let tdid = yaml["rso-authenticator"]["tdid"]["value"]
        .as_str()
        .map(|s| s.to_string());

    Ok(SessionTokens {
        id_token,
        refresh_token,
        tdid,
    })
}

fn decode_game_id_from_id_token(id_token: &str) -> Result<String, String> {
    let payload_b64 = id_token
        .split('.')
        .nth(1)
        .ok_or_else(|| "Invalid id_token format".to_string())?;

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
        .map_err(|e| format!("Failed to decode id_token: {}", e))?;

    let payload: Value = serde_json::from_slice(&payload_bytes).map_err(|e| e.to_string())?;

    if payload["sub"].as_str().is_none() {
        return Err("No puuid in id_token".into());
    }

    let game_name = payload["acct"]["game_name"].as_str().unwrap_or("Unknown");
    let tag_line = payload["acct"]["tag_line"].as_str().unwrap_or("0000");

    Ok(format!("{}#{}", game_name, tag_line))
}

fn get_auth_tokens(tokens: &SessionTokens) -> Result<(String, String), String> {
    // Strategy 1: Use the refresh_token via the OAuth2 token endpoint
    let form_body = format!(
        "grant_type=refresh_token&client_id=riot-client&refresh_token={}",
        tokens.refresh_token
    );
    if let Ok(mut resp) = CLIENT
        .post("https://auth.riotgames.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(&form_body)
    {
        if let Ok(json) = resp.body_mut().read_json::<Value>() {
            if let Some(at) = json["access_token"].as_str() {
                let id_token = json["id_token"]
                    .as_str()
                    .unwrap_or(&tokens.id_token)
                    .to_string();
                return Ok((at.to_string(), id_token));
            }
        }
    }

    // Strategy 2: Cookie-based reauth using tdid
    let body = serde_json::json!({
        "client_id": "play-valorant-web-prod",
        "nonce": "1",
        "redirect_uri": "https://playvalorant.com/opt_in",
        "response_type": "token id_token",
        "scope": "account openid"
    });

    let mut cookie_parts = Vec::new();
    if let Some(tdid) = &tokens.tdid {
        cookie_parts.push(format!("tdid={}", tdid));
    }

    // POST to init an auth session and capture the ssid cookie
    if let Ok(resp) = CLIENT
        .post("https://auth.riotgames.com/api/v1/authorization")
        .header("Content-Type", "application/json")
        .header("Cookie", cookie_parts.join("; "))
        .send_json(&body)
    {
        for val in resp.headers().get_all("set-cookie") {
            if let Ok(val_str) = val.to_str() {
                if val_str.starts_with("ssid=") {
                    if let Some(ssid_val) = val_str.split(';').next() {
                        cookie_parts.push(ssid_val.to_string());
                    }
                }
            }
        }
    }

    // Re-POST with cookies to get access_token via redirect
    let cookie_str = cookie_parts.join("; ");
    let mut resp = CLIENT
        .post("https://auth.riotgames.com/api/v1/authorization")
        .header("Cookie", cookie_str.as_str())
        .header("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|e| e.to_string())?;
    let resp: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;

    if resp["type"].as_str() == Some("response") {
        if let Some(uri) = resp["response"]["parameters"]["uri"].as_str() {
            if let Some(fragment) = uri.split('#').nth(1) {
                let params = fragment
                    .split('&')
                    .filter_map(|p| p.split_once('='))
                    .collect::<HashMap<&str, &str>>();
                if let Some(token) = params.get("access_token") {
                    let id_token = params
                        .get("id_token")
                        .copied()
                        .unwrap_or(tokens.id_token.as_str())
                        .to_string();
                    return Ok((token.to_string(), id_token));
                }
            }
        }
    }

    Err("Session expired. Log in and save session.".into())
}

fn get_entitlements_token(access_token: &str) -> Result<String, String> {
    let auth = format!("Bearer {}", access_token);
    let mut resp = CLIENT
        .post("https://entitlements.auth.riotgames.com/api/token/v1")
        .header("Authorization", auth.as_str())
        .header("Content-Type", "application/json")
        .send_json(&serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    let resp: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;

    resp["entitlements_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No entitlements token in response".to_string())
}

fn get_player_puuid(access_token: &str) -> Result<String, String> {
    let auth = format!("Bearer {}", access_token);
    let mut resp = CLIENT
        .post("https://auth.riotgames.com/userinfo")
        .header("Authorization", auth.as_str())
        .send_json(&serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    let resp: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;

    resp["sub"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No puuid in userinfo".to_string())
}

fn get_region(access_token: &str, id_token: &str) -> Result<String, String> {
    let auth = format!("Bearer {}", access_token);
    let mut resp = CLIENT
        .put("https://riot-geo.pas.si.riotgames.com/pas/v1/product/valorant")
        .header("Authorization", auth.as_str())
        .header("Content-Type", "application/json")
        .send_json(&serde_json::json!({ "id_token": id_token }))
        .map_err(|e| e.to_string())?;
    let resp: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;

    let live = resp["affinities"]["live"]
        .as_str()
        .ok_or_else(|| "No region affinity in response".to_string())?;

    Ok(match live {
        "latam" | "br" => "na",
        other => other,
    }
    .to_string())
}
