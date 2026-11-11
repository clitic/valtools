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
