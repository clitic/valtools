use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::sync::{LazyLock, OnceLock};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("RiotClient/99.0.0.0000000.0000000 rso-auth (Windows;10;;Professional, x64)")
        .build()
        .expect("Failed to build reqwest client")
});

pub fn client() -> &'static reqwest::Client {
    &CLIENT
}

pub async fn post_json(url: &str, headers: &[(&str, &str)], body: Value) -> AppResult<Value> {
    let mut req = CLIENT.post(url);
    for &(k, v) in headers {
        req = req.header(k, v);
    }
    let resp = req.json(&body).send().await?;
    let json = resp.json::<Value>().await?;
    Ok(json)
}

pub async fn get_json(url: &str, headers: &[(&str, &str)]) -> AppResult<Value> {
    let mut req = CLIENT.get(url);
    for &(k, v) in headers {
        req = req.header(k, v);
    }
    let resp = req.send().await?;
    let json = resp.json::<Value>().await?;
    Ok(json)
}

/// Download raw bytes from a URL using the shared reqwest Client.
pub async fn download_bytes(url: &str) -> Option<Vec<u8>> {
    let resp = CLIENT.get(url).send().await.ok()?;
    resp.bytes().await.ok().map(|b| b.to_vec())
}

async fn get_client_version() -> AppResult<String> {
    let resp = get_json("https://valorant-api.com/v1/version", &[]).await?;
    resp["data"]["riotClientVersion"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Api("No client version found".into()))
}

static CLIENT_VERSION: OnceLock<String> = OnceLock::new();

pub async fn cached_client_version() -> AppResult<String> {
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

pub async fn get_storefront(
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

pub async fn get_wallet(
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
