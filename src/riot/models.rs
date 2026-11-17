use std::path::PathBuf;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessoryType {
    Buddy,
    Card,
    Spray,
    Title,
    Unknown,
}

impl AccessoryType {
    pub fn from_item_type_id(uuid: &str) -> Self {
        match uuid.to_lowercase().as_str() {
            "dd3bf334-87f3-40bd-b043-682a57a8dc3a" => Self::Buddy,
            "3f296c07-64c3-494c-923b-fe692a4fa1bd" => Self::Card,
            "d5f120f8-ff8c-4aac-92ea-f2b5acbe9475"
            | "d1c71d47-4436-4a7e-bf9e-f71coupon"
            | "ded352c1-424a-4c28-97ec-03774659b6d8" => Self::Spray,
            "de7caa6b-adf7-4588-bbd1-143831e786c6" | "de7caa6b-adf7-4cc8-abdc-f33e1474a42f" => {
                Self::Title
            }
            _ => Self::Unknown,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Buddy => "Gun Buddy",
            Self::Card => "Player Card",
            Self::Spray => "Spray",
            Self::Title => "Player Title",
            Self::Unknown => "Accessory",
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
pub struct AccessoryOffer {
    pub name: String,
    pub cost: u32,
    pub icon_path: Option<PathBuf>,
    pub item_type: AccessoryType,
}

#[derive(Debug, Clone)]
pub struct AccountStore {
    pub vp: u32,
    pub rp: u32,
    pub kc: u32,
    pub skins: Vec<SkinOffer>,
    pub reset_secs: u64,
    pub accessories: Vec<AccessoryOffer>,
    pub accessory_reset_secs: u64,
}
