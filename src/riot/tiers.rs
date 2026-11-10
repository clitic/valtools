use super::data;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinTier {
    Select,
    Deluxe,
    Premium,
    Exclusive,
    Ultra,
}

impl SkinTier {
    pub fn from_index(idx: u8) -> Option<Self> {
        match idx {
            0 => Some(Self::Select),
            1 => Some(Self::Deluxe),
            2 => Some(Self::Premium),
            3 => Some(Self::Exclusive),
            4 => Some(Self::Ultra),
            _ => None,
        }
    }

    pub fn from_cost(cost: u32) -> Self {
        match cost {
            0..=875 => Self::Select,
            876..=1275 => Self::Deluxe,
            1276..=1775 => Self::Premium,
            1776..=2474 => Self::Exclusive,
            2475 => Self::Ultra,
            _ => {
                if cost == 1750 {
                    Self::Select
                } else if cost == 2550 {
                    Self::Deluxe
                } else if cost == 3550 {
                    Self::Premium
                } else {
                    Self::Exclusive
                }
            }
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

pub fn get_tier_for_skin(skin_uuid: &str, cost: u32) -> SkinTier {
    if let Some(&tier_idx) = data::TIERS_MAP.get(skin_uuid) {
        if let Some(tier) = SkinTier::from_index(tier_idx) {
            return tier;
        }
    }
    SkinTier::from_cost(cost)
}
