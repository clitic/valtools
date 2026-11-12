use crate::{
    anyhow,
    error::Result,
    globals::{CLIENT, RIOT_CLIENT_PLATFORM, RIOT_CLIENT_VERSION, VALTOOLS_DOWNLOADS_DIR},
    riot::auth::{self, RiotAuth},
};
use serde_json::Value;
use std::path::{Path, PathBuf};

const VP_CURRENCY_ID: &str = "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741";
const RP_CURRENCY_ID: &str = "e59aa87c-4cbf-517a-5983-6e81511be9b7";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinTier {
    Select,
    Deluxe,
    Premium,
    Exclusive,
    Ultra,
}

impl SkinTier {
    pub fn from_cost(cost: u32) -> Self {
        match cost {
            0..=875 | 1750 => Self::Select,
            876..=1275 | 2550 => Self::Deluxe,
            1276..=1775 | 3550 => Self::Premium,
            2475 => Self::Ultra,
            _ => Self::Exclusive,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Select => "Select Edition",
            Self::Deluxe => "Deluxe Edition",
            Self::Premium => "Premium Edition",
            Self::Exclusive => "Exclusive Edition",
            Self::Ultra => "Ultra Edition",
        }
    }

    pub fn color_hex(&self) -> u32 {
        match self {
            Self::Select => 0x5a9fe2,
            Self::Deluxe => 0x009587,
            Self::Premium => 0xd1548d,
            Self::Exclusive => 0xf5955b,
            Self::Ultra => 0xfad663,
        }
    }

    pub fn icon_asset(&self) -> &'static str {
        match self {
            Self::Select => "icons/tier-select.png",
            Self::Deluxe => "icons/tier-deluxe.png",
            Self::Premium => "icons/tier-premium.png",
            Self::Exclusive => "icons/tier-exclusive.png",
            Self::Ultra => "icons/tier-ultra.png",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkinOffer {
    pub name: String,
    pub cost: u32,
    pub icon_path: Option<PathBuf>,
    pub tier: SkinTier,
}

#[derive(Debug, Clone)]
pub struct AccountStore {
    pub vp: u32,
    pub rp: u32,
    pub skins: Vec<SkinOffer>,
    pub reset_secs: u64,
}

/// Fetch daily store offers and wallet balances for the saved Riot session file.
pub fn fetch_store(yaml_path: &Path) -> Result<AccountStore> {
    let session = auth::authenticate(yaml_path)?;
    let store_data = get_storefront(&session)?;
    let (vp, rp) = get_wallet(&session)?;

    let layout = &store_data["SkinsPanelLayout"];
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
            let (name, icon_url) = get_skin_info(uuid);
            let icon_path = download_skin_image(&icon_url);
            SkinOffer {
                name,
                cost,
                icon_path,
                tier: SkinTier::from_cost(cost),
            }
        })
        .collect();

    Ok(AccountStore {
        vp,
        rp,
        skins,
        reset_secs,
    })
}

fn with_pd_headers<B>(req: ureq::RequestBuilder<B>, session: &RiotAuth) -> ureq::RequestBuilder<B> {
    let bearer = format!("Bearer {}", session.access_token);
    req.header("Authorization", bearer.as_str())
        .header("X-Riot-Entitlements-JWT", session.entitlements.as_str())
        .header("X-Riot-ClientVersion", RIOT_CLIENT_VERSION.as_str())
        .header("X-Riot-ClientPlatform", RIOT_CLIENT_PLATFORM.as_str())
}

fn get_storefront(session: &RiotAuth) -> Result<Value> {
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

fn get_wallet(session: &RiotAuth) -> Result<(u32, u32)> {
    let url = format!(
        "https://pd.{}.a.pvp.net/store/v1/wallet/{}",
        session.region, session.puuid
    );

    let mut resp = with_pd_headers(CLIENT.get(&url), session)
        .call()
        .map_err(|_| anyhow!("Failed to fetch wallet balance"))?;

    let resp: Value = resp
        .body_mut()
        .read_json()
        .map_err(|_| anyhow!("Invalid wallet response"))?;
    let vp = resp["Balances"][VP_CURRENCY_ID].as_u64().unwrap_or(0) as u32;
    let rp = resp["Balances"][RP_CURRENCY_ID].as_u64().unwrap_or(0) as u32;

    Ok((vp, rp))
}

fn get_skin_info(skin_uuid: &str) -> (String, String) {
    let url = format!(
        "https://valorant-api.com/v1/weapons/skinlevels/{}",
        skin_uuid
    );

    if let Ok(mut resp) = CLIENT.get(&url).call() {
        if let Ok(json) = resp.body_mut().read_json::<Value>() {
            let name = json["data"]["displayName"]
                .as_str()
                .unwrap_or("Unknown Skin")
                .to_string();
            let icon_url = json["data"]["displayIcon"]
                .as_str()
                .unwrap_or("")
                .to_string();
            return (name, icon_url);
        }
    }

    ("Unknown Skin".to_string(), String::new())
}

fn download_skin_image(url: &str) -> Option<PathBuf> {
    if url.is_empty() {
        return None;
    }

    // https://media.valorant-api.com/weaponskinlevels/{uuid}/displayicon.png
    let uuid = url.rsplit('/').nth(1).unwrap_or("unknown");
    let path = VALTOOLS_DOWNLOADS_DIR.join(format!("{}.png", uuid));
    if path.exists() {
        return Some(path);
    }

    let mut resp = CLIENT.get(url).call().ok()?;
    let bytes = resp.body_mut().read_to_vec().ok()?;
    std::fs::write(&path, &bytes).ok()?;
    Some(path)
}
