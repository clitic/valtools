use crate::globals::{CLIENT, RIOT_CLIENT_PLATFORM, RIOT_CLIENT_VERSION};
use serde_json::Value;

pub fn get_storefront(
    access_token: &str,
    entitlements: &str,
    puuid: &str,
    region: &str,
) -> Result<Value, String> {
    let url = format!(
        "https://pd.{}.a.pvp.net/store/v3/storefront/{}",
        region, puuid
    );
    let auth = format!("Bearer {}", access_token);

    let mut resp = CLIENT
        .post(&url)
        .header("Authorization", auth.as_str())
        .header("X-Riot-Entitlements-JWT", entitlements)
        .header("X-Riot-ClientVersion", RIOT_CLIENT_VERSION.as_str())
        .header("X-Riot-ClientPlatform", RIOT_CLIENT_PLATFORM.as_str())
        .header("Content-Type", "application/json")
        .send_json(&serde_json::json!({}))
        .map_err(|e| e.to_string())?;

    resp.body_mut().read_json().map_err(|e| e.to_string())
}

pub fn get_wallet(
    access_token: &str,
    entitlements: &str,
    puuid: &str,
    region: &str,
) -> Result<(u32, u32), String> {
    let url = format!("https://pd.{}.a.pvp.net/store/v1/wallet/{}", region, puuid);
    let auth = format!("Bearer {}", access_token);

    let mut resp = CLIENT
        .get(&url)
        .header("Authorization", auth.as_str())
        .header("X-Riot-Entitlements-JWT", entitlements)
        .header("X-Riot-ClientVersion", RIOT_CLIENT_VERSION.as_str())
        .header("X-Riot-ClientPlatform", RIOT_CLIENT_PLATFORM.as_str())
        .call()
        .map_err(|e| e.to_string())?;

    let resp: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;

    let vp = resp["Balances"]["85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741"]
        .as_u64()
        .unwrap_or(0) as u32;

    let rp = resp["Balances"]["e59aa87c-4cbf-517a-5983-6e81511be9b7"]
        .as_u64()
        .unwrap_or(0) as u32;

    Ok((vp, rp))
}
