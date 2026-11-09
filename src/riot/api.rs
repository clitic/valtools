use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::Path,
    sync::{LazyLock, OnceLock},
};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("RiotClient/99.0.0.0000000.0000000 rso-auth (Windows;10;;Professional, x64)")
        .build()
        .expect("Failed to build reqwest client")
});

pub(crate) async fn post_json(
    url: &str,
    headers: &[(&str, &str)],
    body: Value,
) -> AppResult<Value> {
    let mut req = CLIENT.post(url);
    for &(k, v) in headers {
        req = req.header(k, v);
    }
    let resp = req.json(&body).send().await?;
    let json = resp.json::<Value>().await?;
    Ok(json)
}

pub(crate) async fn get_json(url: &str, headers: &[(&str, &str)]) -> AppResult<Value> {
    let mut req = CLIENT.get(url);
    for &(k, v) in headers {
        req = req.header(k, v);
    }
    let resp = req.send().await?;
    let json = resp.json::<Value>().await?;
    Ok(json)
}

/// Download raw bytes from a URL using the shared reqwest Client.
pub(crate) async fn download_bytes(url: &str) -> Option<Vec<u8>> {
    let resp = CLIENT.get(url).send().await.ok()?;
    resp.bytes().await.ok().map(|b| b.to_vec())
}

/// Tokens extracted from the new RiotGamesPrivateSettings.yaml structure.
#[derive(Clone, Debug)]
pub(crate) struct SessionTokens {
    pub id_token: String,
    pub refresh_token: String,
    pub tdid: Option<String>,
}

pub(crate) fn extract_session_tokens(yaml_path: &Path) -> AppResult<SessionTokens> {
    let content = std::fs::read_to_string(yaml_path)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let riot_client = &yaml["psl"]["authorization"]["riot-client"];

    let id_token = riot_client["id_token"]
        .as_str()
        .ok_or(AppError::Api("No id_token found in settings".into()))?
        .to_string();

    let refresh_token = riot_client["refresh_token"]
        .as_str()
        .ok_or(AppError::Api("No refresh_token found in settings".into()))?
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
pub(crate) fn get_player_info_from_id_token(id_token: &str) -> AppResult<(String, String)> {
    let parts = id_token.split('.').collect::<Vec<&str>>();
    if parts.len() < 2 {
        return Err(AppError::Api("Invalid id_token format".into()));
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
        .map_err(|e| AppError::Api(format!("Failed to decode id_token payload: {}", e)))?;

    let payload: Value = serde_json::from_slice(&payload_bytes)?;

    let game_name = payload["acct"]["game_name"]
        .as_str()
        .unwrap_or("Unknown");
    let tag_line = payload["acct"]["tag_line"]
        .as_str()
        .unwrap_or("0000");
    let puuid = payload["sub"]
        .as_str()
        .ok_or(AppError::Api("No sub (puuid) in id_token".into()))?;

    Ok((format!("{}#{}", game_name, tag_line), puuid.to_string()))
}

pub(crate) async fn get_access_token(tokens: &SessionTokens) -> AppResult<String> {
    // Strategy 1: Use the refresh_token via the OAuth2 token endpoint
    let form_body = format!(
        "grant_type=refresh_token&client_id=riot-client&refresh_token={}",
        tokens.refresh_token
    );
    if let Ok(resp) = CLIENT
        .post("https://auth.riotgames.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_body)
        .send()
        .await
    {
        if let Ok(json) = resp.json::<Value>().await {
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
    if let Ok(resp) = CLIENT
        .post("https://auth.riotgames.com/api/v1/authorization")
        .header("Content-Type", "application/json")
        .header("Cookie", cookie_parts.join("; "))
        .json(&body)
        .send()
        .await
    {
        for val in resp.headers().get_all(reqwest::header::SET_COOKIE) {
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
    let resp = post_json(
        "https://auth.riotgames.com/api/v1/authorization",
        &[
            ("Cookie", cookie_str.as_str()),
            ("Content-Type", "application/json"),
        ],
        body,
    )
    .await?;

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

    Err(AppError::Api("Session expired. Re-login required.".into()))
}

pub(crate) async fn get_entitlements_token(access_token: &str) -> AppResult<String> {
    let auth = format!("Bearer {}", access_token);
    let resp = post_json(
        "https://entitlements.auth.riotgames.com/api/token/v1",
        &[
            ("Authorization", auth.as_str()),
            ("Content-Type", "application/json"),
        ],
        serde_json::json!({}),
    )
    .await?;

    resp["entitlements_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Api("No entitlements_token in response".into()))
}

pub(crate) async fn get_player_info(access_token: &str) -> AppResult<(String, String)> {
    let auth = format!("Bearer {}", access_token);
    let resp = post_json(
        "https://auth.riotgames.com/userinfo",
        &[("Authorization", auth.as_str())],
        serde_json::json!({}),
    )
    .await?;

    let game_name = resp["acct"]["game_name"]
        .as_str()
        .unwrap_or("Unknown");
    let tag_line = resp["acct"]["tag_line"].as_str().unwrap_or("0000");
    let puuid = resp["sub"]
        .as_str()
        .ok_or(AppError::Api("No puuid in userinfo".into()))?;

    Ok((format!("{}#{}", game_name, tag_line), puuid.to_string()))
}

async fn get_client_version() -> AppResult<String> {
    let resp = get_json("https://valorant-api.com/v1/version", &[]).await?;
    resp["data"]["riotClientVersion"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Api("No client version found".into()))
}

static CLIENT_VERSION: OnceLock<String> = OnceLock::new();

pub(crate) async fn cached_client_version() -> AppResult<String> {
    if let Some(v) = CLIENT_VERSION.get() {
        return Ok(v.clone());
    }
    let v = get_client_version().await?;
    Ok(CLIENT_VERSION.get_or_init(|| v).clone())
}

fn platform_encoded() -> String {
    let platform = serde_json::json!({
        "platformType": "PC",
        "platformOS": "Windows",
        "platformOSVersion": "10.0.19042.1.256.64bit",
        "platformChipset": "Unknown"
    });
    STANDARD.encode(platform.to_string())
}

pub(crate) async fn get_storefront(
    access_token: &str,
    entitlements: &str,
    puuid: &str,
    client_version: &str,
    region: &str,
) -> AppResult<Value> {
    let url = format!(
        "https://pd.{}.a.pvp.net/store/v3/storefront/{}",
        region, puuid
    );
    let auth = format!("Bearer {}", access_token);
    let plat = platform_encoded();

    post_json(
        &url,
        &[
            ("Authorization", auth.as_str()),
            ("X-Riot-Entitlements-JWT", entitlements),
            ("X-Riot-ClientVersion", client_version),
            ("X-Riot-ClientPlatform", plat.as_str()),
            ("Content-Type", "application/json"),
        ],
        serde_json::json!({}),
    )
    .await
}

pub(crate) async fn get_wallet(
    access_token: &str,
    entitlements: &str,
    puuid: &str,
    client_version: &str,
    region: &str,
) -> AppResult<(u32, u32)> {
    let url = format!(
        "https://pd.{}.a.pvp.net/store/v1/wallet/{}",
        region, puuid
    );
    let auth = format!("Bearer {}", access_token);
    let plat = platform_encoded();

    let resp = get_json(
        &url,
        &[
            ("Authorization", auth.as_str()),
            ("X-Riot-Entitlements-JWT", entitlements),
            ("X-Riot-ClientVersion", client_version),
            ("X-Riot-ClientPlatform", plat.as_str()),
        ],
    )
    .await?;

    let vp = resp["Balances"]["85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741"]
        .as_u64()
        .unwrap_or(0) as u32;

    let rp = resp["Balances"]["e59aa87c-4cbf-517a-5983-6e81511be9b7"]
        .as_u64()
        .unwrap_or(0) as u32;

    Ok((vp, rp))
}
