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

pub(crate) fn extract_ssid(yaml_path: &Path) -> AppResult<String> {
    let content = std::fs::read_to_string(yaml_path)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let cookies = yaml["riot-login"]["persist"]["session"]["cookies"]
        .as_sequence()
        .ok_or(AppError::Api("No session cookies found in settings".into()))?;

    for cookie in cookies {
        if cookie["name"].as_str() == Some("ssid") {
            return cookie["value"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::Api("ssid cookie has no value".into()));
        }
    }

    Err(AppError::Api("ssid cookie not found in session".into()))
}

pub(crate) fn get_access_token(ssid: &str) -> AppResult<String> {
    let body = serde_json::json!({
        "client_id": "play-valorant-web-prod",
        "nonce": "1",
        "redirect_uri": "https://playvalorant.com/opt_in",
        "response_type": "token id_token",
        "scope": "account openid"
    });

    let cookie = format!("ssid={}", ssid);
    let resp = post_json(
        "https://auth.riotgames.com/api/v1/authorization",
        &[
            ("Cookie", cookie.as_str()),
            ("Content-Type", "application/json"),
        ],
        body,
    )?;

    if resp["type"].as_str() != Some("response") {
        return Err(AppError::Api("Session expired. Re-login required.".into()));
    }

    let uri = resp["response"]["parameters"]["uri"]
        .as_str()
        .ok_or(AppError::Api("No redirect URI in auth response".into()))?;

    let fragment = uri
        .split('#')
        .nth(1)
        .ok_or(AppError::Api("No fragment in redirect URI".into()))?;
    let params = fragment
        .split('&')
        .filter_map(|p| p.split_once('='))
        .collect::<HashMap<&str, &str>>();

    params
        .get("access_token")
        .map(|t| t.to_string())
        .ok_or_else(|| AppError::Api("No access_token in redirect URI".into()))
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
