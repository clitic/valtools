use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::sync::{LazyLock, OnceLock};

static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| ureq::Agent::new_with_defaults());

pub fn agent() -> &'static ureq::Agent {
    &AGENT
}

pub fn post_json(url: &str, headers: &[(&str, &str)], body: Value) -> AppResult<Value> {
    let mut req = AGENT.post(url);
    for &(k, v) in headers {
        req = req.header(k, v);
    }
    let mut resp = req.send_json(&body)?;
    let json: Value = resp.body_mut().read_json()?;
    Ok(json)
}

pub fn get_json(url: &str, headers: &[(&str, &str)]) -> AppResult<Value> {
    let mut req = AGENT.get(url);
    for &(k, v) in headers {
        req = req.header(k, v);
    }
    let mut resp = req.call()?;
    let json: Value = resp.body_mut().read_json()?;
    Ok(json)
}

/// Download raw bytes from a URL using the shared ureq Agent.
pub fn download_bytes(url: &str) -> Option<Vec<u8>> {
    let mut resp = AGENT.get(url).call().ok()?;
    resp.body_mut().read_to_vec().ok()
}

fn get_client_version() -> AppResult<String> {
    let resp = get_json("https://valorant-api.com/v1/version", &[])?;
    resp["data"]["riotClientVersion"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Api("No client version found".into()))
}

static CLIENT_VERSION: OnceLock<String> = OnceLock::new();

pub fn cached_client_version() -> AppResult<String> {
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

pub fn get_storefront(
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

pub fn get_wallet(
    access_token: &str,
    entitlements: &str,
    puuid: &str,
    client_version: &str,
    region: &str,
) -> AppResult<(u32, u32)> {
    let url = format!("https://pd.{}.a.pvp.net/store/v1/wallet/{}", region, puuid);
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

    let rp = resp["Balances"]["e59aa87c-4cbf-517a-5983-6e81511be9b7"]
        .as_u64()
        .unwrap_or(0) as u32;

    Ok((vp, rp))
}
