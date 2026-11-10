use gpui_kit::component::{
    alert::Alert, tag::Tag, ActiveTheme as _, Icon, Sizable as _, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, App, AppContext as _, AsyncApp, Context, Entity, IntoElement, ParentElement as _,
    Render, Styled as _, WeakEntity, Window,
};

use super::card::{render_skeleton_card, render_skin_card};
use crate::riot::{
    api, auth,
    store::{self, AccountStore},
    tiers,
};

pub struct ShopDialogView {
    loading: bool,
    store: Option<AccountStore>,
    error: Option<String>,
}

impl ShopDialogView {
    async fn fetch_flow(label: String, this: WeakEntity<Self>, cx: &mut AsyncApp) {
        let yaml_path = crate::session::ACCOUNTS_DIR
            .join(&label)
            .join("RiotGamesPrivateSettings.yaml");

        if !yaml_path.exists() {
            this.update(cx, |this, cx| {
                this.loading = false;
                this.error = Some(format!(
                    "No saved session found for '{}'. Please log in via Riot Client and click 'Save Session' in ValTools.",
                    label
                ));
                cx.notify();
            })
            .ok();
            return;
        }

        let store_res = cx
            .background_executor()
            .spawn(async move {
                let tokens = auth::extract_session_tokens(&yaml_path)?;
                let access_token = auth::get_access_token(&tokens)?;
                let entitlements = auth::get_entitlements_token(&access_token)?;
                let (_game_id, puuid) = auth::get_player_info(&access_token)?;
                let client_version = api::cached_client_version()?;
                let store_data = api::get_storefront(
                    &access_token,
                    &entitlements,
                    &puuid,
                    &client_version,
                    "ap",
                )?;
                let (vp, rp) =
                    api::get_wallet(&access_token, &entitlements, &puuid, &client_version, "ap")?;

                let offers = store_data["SkinsPanelLayout"]["SingleItemStoreOffers"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();

                let reset_secs = store_data["SkinsPanelLayout"]
                    ["SingleItemOffersRemainingDurationInSeconds"]
                    .as_u64()
                    .unwrap_or(0);

                let mut skins = Vec::new();
                for offer in offers {
                    let uuid = offer["OfferID"].as_str().unwrap_or("").to_string();
                    let cost = offer["Cost"]["85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741"]
                        .as_u64()
                        .unwrap_or(0) as u32;

                    let (name, icon_url) = store::get_skin_info(&uuid);
                    let icon_path = store::download_single_image(&icon_url);
                    let tier = tiers::get_tier_for_skin(&uuid, cost);
                    skins.push(crate::riot::SkinOffer {
                        name,
                        cost,
                        icon_path,
                        tier,
                    });
                }

                Ok::<AccountStore, crate::error::AppError>(AccountStore {
                    vp,
                    rp,
                    skins,
                    reset_secs,
                })
            })
            .await;

        match store_res {
            Ok(store) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.store = Some(store);
                    cx.notify();
                })
                .ok();
            }
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
            }
        }
    }

    pub fn new(label: String, cx: &mut Context<Self>) -> Self {
        let label_clone = label.clone();
        cx.spawn(async move |this, cx| {
            Self::fetch_flow(label_clone, this, cx).await;
        })
        .detach();

        Self {
            loading: true,
            store: None,
            error: None,
        }
    }

    pub fn open(label: String, window: &mut Window, cx: &mut App) {
        let title = format!("{}'s Daily Shop", label);
        let shop_view: Entity<Self> = cx.new(|cx| Self::new(label, cx));

        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title(title.clone())
                .width(px(540.0))
                .child(shop_view.clone())
        });
    }
}

impl Render for ShopDialogView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .w_full()
            .gap_3()
            .child(if self.loading {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    // Skeletal Chips row just below title
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                gpui_kit::component::skeleton::Skeleton::new()
                                    .h(px(24.0))
                                    .w(px(72.0))
                                    .rounded_full(),
                            )
                            .child(
                                gpui_kit::component::skeleton::Skeleton::new()
                                    .h(px(24.0))
                                    .w(px(72.0))
                                    .rounded_full(),
                            )
                            .child(
                                gpui_kit::component::skeleton::Skeleton::new()
                                    .h(px(24.0))
                                    .w(px(108.0))
                                    .rounded_full(),
                            ),
                    )
                    // Skeletal 2x2 Grid
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(render_skeleton_card(0, theme))
                                    .child(render_skeleton_card(1, theme)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(render_skeleton_card(2, theme))
                                    .child(render_skeleton_card(3, theme)),
                            ),
                    )
                    .into_any_element()
            } else if let Some(err) = &self.error {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(Alert::error("shop_error", err.clone()).title("Failed to load shop"))
                    .into_any_element()
            } else if let Some(store) = &self.store {
                let s1 = store.skins.get(0);
                let s2 = store.skins.get(1);
                let s3 = store.skins.get(2);
                let s4 = store.skins.get(3);

                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    // Chips row just below title (using official Tag component)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            // VP Chip
                            .child(
                                Tag::new()
                                    .rounded_full()
                                    .gap_1p5()
                                    .child(Icon::new(gpui_kit::assets::IconName::Coins).xsmall())
                                    .child(format!("{} VP", store.vp)),
                            )
                            // RP Chip
                            .child(
                                Tag::new()
                                    .rounded_full()
                                    .gap_1p5()
                                    .child(Icon::new(gpui_kit::assets::IconName::Coins).xsmall())
                                    .child(format!("{} RP", store.rp)),
                            )
                            // Resets in Chip
                            .child(
                                Tag::new()
                                    .rounded_full()
                                    .gap_1p5()
                                    .child(Icon::new(gpui_kit::assets::IconName::Clock).xsmall())
                                    .child(format!(
                                        "Resets in {}",
                                        store::format_reset_time(store.reset_secs)
                                    )),
                            ),
                    )
                    // Grid 2x2
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .when_some(s1, |this, s| {
                                        this.child(render_skin_card(s, 0, theme))
                                    })
                                    .when_some(s2, |this, s| {
                                        this.child(render_skin_card(s, 1, theme))
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .when_some(s3, |this, s| {
                                        this.child(render_skin_card(s, 2, theme))
                                    })
                                    .when_some(s4, |this, s| {
                                        this.child(render_skin_card(s, 3, theme))
                                    }),
                            ),
                    )
                    .into_any_element()
            } else {
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("No store data")
                    .into_any_element()
            })
    }
}
