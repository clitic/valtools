use crate::{
    anyhow,
    error::Result,
    globals::{CLIENT, RIOT_CLIENT_PLATFORM, RIOT_CLIENT_VERSION},
    riot::{
        auth::{self, RiotAuth},
        models::AccountStore,
        skins,
    },
};
use serde_json::Value;
use std::path::Path;

const VP_CURRENCY_ID: &str = "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741";
const RP_CURRENCY_ID: &str = "e59aa87c-4cbf-517a-5983-6e81511be9b7";

/// Fetch daily store offers and wallet balances for the saved Riot session file.
pub fn fetch_store(yaml_path: &Path) -> Result<AccountStore> {
    let session = auth::authenticate(yaml_path)?;
    let storefront = fetch_storefront(&session)?;
    let (vp, rp) = fetch_wallet(&session)?;

    Ok(parse_storefront(&storefront, vp, rp))
}

fn parse_storefront(storefront: &Value, vp: u32, rp: u32) -> AccountStore {
    let layout = &storefront["SkinsPanelLayout"];
    let offers = layout["SingleItemStoreOffers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let reset_secs = layout["SingleItemOffersRemainingDurationInSeconds"]
        .as_u64()
        .unwrap_or(0);

    let skins = offers
        .into_iter()
        .map(|offer| {
            let uuid = offer["OfferID"].as_str().unwrap_or("");
            let cost = offer["Cost"][VP_CURRENCY_ID].as_u64().unwrap_or(0) as u32;
            skins::resolve_skin_offer(uuid, cost)
        })
        .collect();

    AccountStore {
        vp,
        rp,
        skins,
        reset_secs,
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

fn fetch_wallet(session: &RiotAuth) -> Result<(u32, u32)> {
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

    Ok((vp, rp))
}
