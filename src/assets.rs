use gpui_kit::{AssetSource, Result, SharedString};
use std::borrow::Cow;

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match path {
            "icons/github.svg" => Ok(Some(Cow::Borrowed(include_bytes!("../icons/github.svg")))),
            "icons/riot.svg" => Ok(Some(Cow::Borrowed(include_bytes!("../icons/riot.svg")))),
            "icons/valorant.svg" => {
                Ok(Some(Cow::Borrowed(include_bytes!("../icons/valorant.svg"))))
            }
            "icons/tier-select.png" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../icons/tier-select.png"
            )))),
            "icons/tier-deluxe.png" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../icons/tier-deluxe.png"
            )))),
            "icons/tier-premium.png" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../icons/tier-premium.png"
            )))),
            "icons/tier-exclusive.png" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../icons/tier-exclusive.png"
            )))),
            "icons/tier-ultra.png" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../icons/tier-ultra.png"
            )))),
            _ => gpui_kit::assets::AllAssets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut list = gpui_kit::assets::AllAssets.list(path).unwrap_or_default();
        if path == "icons" || path.is_empty() {
            list.push("icons/github.svg".into());
            list.push("icons/riot.svg".into());
            list.push("icons/valorant.svg".into());
            list.push("icons/tier-select.png".into());
            list.push("icons/tier-deluxe.png".into());
            list.push("icons/tier-premium.png".into());
            list.push("icons/tier-exclusive.png".into());
            list.push("icons/tier-ultra.png".into());
        }
        Ok(list)
    }
}
