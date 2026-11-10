use super::api;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{collections::HashMap, path::Path};

/// Tokens extracted from RiotGamesPrivateSettings.yaml.
#[derive(Clone, Debug)]
pub struct SessionTokens {
    pub id_token: String,
    pub refresh_token: String,
    pub tdid: Option<String>,
}

pub fn extract_session_tokens(yaml_path: &Path) -> Result<SessionTokens, String> {
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

/// Decode the id_token JWT to extract player info without an API call.
pub fn get_player_info_from_id_token(id_token: &str) -> Result<(String, String), String> {
    let parts = id_token.split('.').collect::<Vec<&str>>();
    if parts.len() < 2 {
        return Err("Invalid id_token format".into());
    }

    // Decode the payload (second part) with padding tolerance
    let payload_b64 = parts[1];
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

    let game_name = payload["acct"]["game_name"].as_str().unwrap_or("Unknown");
    let tag_line = payload["acct"]["tag_line"].as_str().unwrap_or("0000");
    let puuid = payload["sub"]
        .as_str()
        .ok_or_else(|| "No puuid in id_token".to_string())?;

    Ok((format!("{}#{}", game_name, tag_line), puuid.to_string()))
}

pub fn get_access_token(tokens: &SessionTokens) -> Result<String, String> {
    // Strategy 1: Use the refresh_token via the OAuth2 token endpoint
    let form_body = format!(
        "grant_type=refresh_token&client_id=riot-client&refresh_token={}",
        tokens.refresh_token
    );
    if let Ok(mut resp) = api::agent()
        .post("https://auth.riotgames.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(&form_body)
    {
        if let Ok(json) = resp.body_mut().read_json::<Value>() {
            if let Some(at) = json["access_token"].as_str() {
                return Ok(at.to_string());
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
    if let Ok(resp) = api::agent()
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
    let resp = api::post_json(
        "https://auth.riotgames.com/api/v1/authorization",
        &[
            ("Cookie", cookie_str.as_str()),
            ("Content-Type", "application/json"),
        ],
        body,
    )?;

    if resp["type"].as_str() == Some("response") {
        if let Some(uri) = resp["response"]["parameters"]["uri"].as_str() {
            if let Some(fragment) = uri.split('#').nth(1) {
                let params = fragment
                    .split('&')
                    .filter_map(|p| p.split_once('='))
                    .collect::<HashMap<&str, &str>>();
                if let Some(token) = params.get("access_token") {
                    return Ok(token.to_string());
                }
            }
        }
    }

    Err("Session expired. Log in and save session.".into())
}

pub fn get_entitlements_token(access_token: &str) -> Result<String, String> {
    let auth = format!("Bearer {}", access_token);
    let resp = api::post_json(
        "https://entitlements.auth.riotgames.com/api/token/v1",
        &[
            ("Authorization", auth.as_str()),
            ("Content-Type", "application/json"),
        ],
        serde_json::json!({}),
    )?;

    resp["entitlements_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No entitlements token in response".to_string())
}

pub fn get_player_info(access_token: &str) -> Result<(String, String), String> {
    let auth = format!("Bearer {}", access_token);
    let resp = api::post_json(
        "https://auth.riotgames.com/userinfo",
        &[("Authorization", auth.as_str())],
        serde_json::json!({}),
    )?;

    let game_name = resp["acct"]["game_name"].as_str().unwrap_or("Unknown");
    let tag_line = resp["acct"]["tag_line"].as_str().unwrap_or("0000");
    let puuid = resp["sub"]
        .as_str()
        .ok_or_else(|| "No puuid in userinfo".to_string())?;

    Ok((format!("{}#{}", game_name, tag_line), puuid.to_string()))
}
