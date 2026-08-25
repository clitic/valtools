use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{collections::HashMap, path::Path, sync::{LazyLock, OnceLock}};
use ureq::{Agent, tls::{TlsConfig, TlsProvider, RootCerts}};

static AGENT: LazyLock<Agent> = LazyLock::new(|| {
    Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent()
});

pub(crate) fn post_json(
    url: &str,
    headers: &[(&str, &str)],
    body: Value,
) -> AppResult<Value> {
    let mut req = AGENT.post(url);
    for &(k, v) in headers {
        req = req.header(k, v);
    }
    let mut resp = req.send_json(&body)?;
    let text = resp.body_mut().read_to_string()?;
    Ok(serde_json::from_str(&text)?)
}

pub(crate) fn get_json(url: &str, headers: &[(&str, &str)]) -> AppResult<Value> {
    let mut req = AGENT.get(url);
    for &(k, v) in headers {
        req = req.header(k, v);
    }
    let mut resp = req.call()?;
    let text = resp.body_mut().read_to_string()?;
    Ok(serde_json::from_str(&text)?)
}

/// Download raw bytes from a URL using the shared Agent.
pub(crate) fn download_bytes(url: &str) -> Option<Vec<u8>> {
    let mut resp = AGENT.get(url).call().ok()?;
    resp.body_mut().read_to_vec().ok()
}

/// Tokens extracted from the new RiotGamesPrivateSettings.yaml structure.
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

    Ok(SessionTokens { id_token, refresh_token, tdid })
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
            // Try with padding added
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

pub(crate) fn get_access_token(tokens: &SessionTokens) -> AppResult<String> {
    // Strategy 1: Use the refresh_token via the OAuth2 token endpoint
    let form_body = format!(
        "grant_type=refresh_token&client_id=riot-client&refresh_token={}",
        tokens.refresh_token
    );
    if let Ok(mut resp) = AGENT
        .post("https://auth.riotgames.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(form_body.as_bytes())
    {
        if let Ok(text) = resp.body_mut().read_to_string() {
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                if let Some(at) = json["access_token"].as_str() {
                    return Ok(at.to_string());
                }
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
    if let Ok(mut resp) = AGENT
        .post("https://auth.riotgames.com/api/v1/authorization")
        .header("Content-Type", "application/json")
        .header("Cookie", &cookie_parts.join("; "))
        .send_json(&body)
    {
        let _ = resp.body_mut().read_to_string();
        for cookie_header in resp.headers().get_all("set-cookie") {
            if let Some(val) = cookie_header.to_str().ok() {
                if val.starts_with("ssid=") {
                    if let Some(ssid_val) = val.split(';').next() {
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

    Err(AppError::Api("Session expired. Re-login required.".into()))
}

pub(crate) fn get_entitlements_token(access_token: &str) -> AppResult<String> {
    let auth = format!("Bearer {}", access_token);
    let resp = post_json(
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
        .ok_or_else(|| AppError::Api("No entitlements_token in response".into()))
}

pub(crate) fn get_player_info(access_token: &str) -> AppResult<(String, String)> {
    let auth = format!("Bearer {}", access_token);
    let resp = post_json(
        "https://auth.riotgames.com/userinfo",
        &[("Authorization", auth.as_str())],
        serde_json::json!({}),
    )?;

    let game_name = resp["acct"]["game_name"]
        .as_str()
        .unwrap_or("Unknown");
    let tag_line = resp["acct"]["tag_line"].as_str().unwrap_or("0000");
    let puuid = resp["sub"]
        .as_str()
        .ok_or(AppError::Api("No puuid in userinfo".into()))?;

    Ok((format!("{}#{}", game_name, tag_line), puuid.to_string()))
}

fn get_client_version() -> AppResult<String> {
    let resp = get_json("https://valorant-api.com/v1/version", &[])?;
    resp["data"]["riotClientVersion"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Api("No client version found".into()))
}

static CLIENT_VERSION: OnceLock<String> = OnceLock::new();

pub(crate) fn cached_client_version() -> AppResult<String> {
    if let Some(v) = CLIENT_VERSION.get() {
        return Ok(v.clone());
    }
    let v = get_client_version()?;
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

pub(crate) fn get_storefront(
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
}

pub(crate) fn get_wallet(
    access_token: &str,
    entitlements: &str,
    puuid: &str,
    client_version: &str,
    region: &str,
) -> AppResult<u32> {
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
    )?;

    let vp = resp["Balances"]["85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741"]
        .as_u64()
        .unwrap_or(0) as u32;

    Ok(vp)
}
