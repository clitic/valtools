use crate::{
    anyhow,
    error::Result,
    globals::CLIENT,
    riot::{
        accessories,
        auth::{self, RiotAuth},
        models::AccountStore,
        skins,
    },
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{path::Path, sync::LazyLock};

const VP_CURRENCY_ID: &str = "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741";
const RP_CURRENCY_ID: &str = "e59aa87c-4cbf-517a-5983-6e81511be9b7";
const KC_CURRENCY_ID: &str = "85ca954a-41f2-ce94-9b45-8ca3dd39a00d";

static RIOT_CLIENT_PLATFORM: LazyLock<String> = LazyLock::new(|| {
    STANDARD.encode(
        serde_json::json!({
            "platformType": "PC",
            "platformOS": "Windows",
            "platformOSVersion": "10.0.19042.1.256.64bit",
            "platformChipset": "Unknown"
        })
        .to_string(),
    )
});

static RIOT_CLIENT_VERSION: LazyLock<String> = LazyLock::new(|| {
    let Ok(mut resp) = CLIENT.get("https://valorant-api.com/v1/version").call() else {
        return String::new();
    };
    let Ok(json) = resp.body_mut().read_json::<Value>() else {
        return String::new();
    };
    json["data"]["riotClientVersion"]
        .as_str()
        .unwrap_or("")
        .to_owned()
});

/// Fetch daily store offers, accessories store, and wallet balances for the saved Riot session file.
pub fn fetch_store(yaml_path: &Path) -> Result<AccountStore> {
    let session = auth::authenticate(yaml_path)?;
    let storefront = fetch_storefront(&session)?;
    let (vp, rp, kc) = fetch_wallet(&session)?;

    Ok(parse_storefront(&storefront, vp, rp, kc))
}

fn parse_storefront(storefront: &Value, vp: u32, rp: u32, kc: u32) -> AccountStore {
    let layout = &storefront["SkinsPanelLayout"];
    let offers = layout["SingleItemStoreOffers"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let reset_secs = layout["SingleItemOffersRemainingDurationInSeconds"]
        .as_u64()
        .unwrap_or(0);

    let skins = offers
        .iter()
        .map(|offer| {
            let uuid = offer["OfferID"].as_str().unwrap_or("");
            let cost = offer["Cost"][VP_CURRENCY_ID].as_u64().unwrap_or(0) as u32;
            skins::resolve_skin_offer(uuid, cost)
        })
        .collect();

    let accessory_store = &storefront["AccessoryStore"];
    let accessory_reset_secs = accessory_store["AccessoryStoreRemainingDurationInSeconds"]
        .as_u64()
        .or_else(|| accessory_store["RemainingDurationInSeconds"].as_u64())
        .unwrap_or(0);

    let accessory_offers = accessory_store["AccessoryStoreOffers"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let accessories = accessory_offers
        .iter()
        .filter_map(|entry| {
            let offer = entry.get("Offer").unwrap_or(entry);
            let cost = offer["Cost"][KC_CURRENCY_ID]
                .as_u64()
                .or_else(|| {
                    offer["Cost"]
                        .as_object()
                        .and_then(|m| m.values().next())
                        .and_then(|v| v.as_u64())
                })
                .unwrap_or(0) as u32;

            let rewards = offer["Rewards"].as_array()?;
            let first_reward = rewards.first()?;
            let item_type_id = first_reward["ItemTypeID"].as_str().unwrap_or("");
            let item_id = first_reward["ItemID"].as_str().unwrap_or("");

            if item_id.is_empty() {
                return None;
            }

            Some(accessories::resolve_accessory_offer(
                item_type_id,
                item_id,
                cost,
            ))
        })
        .collect();

    AccountStore {
        vp,
        rp,
        kc,
        skins,
        reset_secs,
        accessories,
        accessory_reset_secs,
    }
}

fn with_pd_headers<B>(req: ureq::RequestBuilder<B>, session: &RiotAuth) -> ureq::RequestBuilder<B> {
    let bearer = format!("Bearer {}", session.access_token);
    req.header("Authorization", bearer.as_str())
        .header("X-Riot-Entitlements-JWT", session.entitlements.as_str())
        .header("X-Riot-ClientVersion", RIOT_CLIENT_VERSION.as_str())
        .header("X-Riot-ClientPlatform", RIOT_CLIENT_PLATFORM.as_str())
}

fn fetch_storefront(session: &RiotAuth) -> Result<Value> {
    let url = format!(
        "https://pd.{}.a.pvp.net/store/v3/storefront/{}",
        session.region, session.puuid
    );

    let mut resp = with_pd_headers(CLIENT.post(&url), session)
        .header("Content-Type", "application/json")
        .send_json(&serde_json::json!({}))
        .map_err(|_| anyhow!("Failed to fetch daily store"))?;

    resp.body_mut()
        .read_json()
        .map_err(|_| anyhow!("Invalid daily store response"))
}

fn fetch_wallet(session: &RiotAuth) -> Result<(u32, u32, u32)> {
    let url = format!(
        "https://pd.{}.a.pvp.net/store/v1/wallet/{}",
        session.region, session.puuid
    );

    let mut resp = with_pd_headers(CLIENT.get(&url), session)
        .call()
        .map_err(|_| anyhow!("Failed to fetch wallet balance"))?;

    let json: Value = resp
        .body_mut()
        .read_json()
        .map_err(|_| anyhow!("Invalid wallet response"))?;
    let vp = json["Balances"][VP_CURRENCY_ID].as_u64().unwrap_or(0) as u32;
    let rp = json["Balances"][RP_CURRENCY_ID].as_u64().unwrap_or(0) as u32;
    let kc = json["Balances"][KC_CURRENCY_ID]
        .as_u64()
        .or_else(|| {
            json["Balances"].as_object().and_then(|map| {
                map.iter().find_map(|(k, v)| {
                    if k.eq_ignore_ascii_case(KC_CURRENCY_ID) {
                        v.as_u64()
                    } else {
                        None
                    }
                })
            })
        })
        .or_else(|| {
            json["Balances"].as_object().and_then(|map| {
                map.iter().find_map(|(k, v)| {
                    if !k.eq_ignore_ascii_case(VP_CURRENCY_ID)
                        && !k.eq_ignore_ascii_case(RP_CURRENCY_ID)
                        && !k.eq_ignore_ascii_case("f08d4ae3-939c-4576-ab26-09ce1f23bb37")
                    {
                        v.as_u64()
                    } else {
                        None
                    }
                })
            })
        })
        .unwrap_or(0) as u32;

    Ok((vp, rp, kc))
}
