use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    alert::Alert,
    button::{Button, ButtonVariants},
    skeleton::Skeleton,
    tag::Tag,
    ActiveTheme as _, ColorName, Icon, Sizable as _, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, img, px, rgb, App, AppContext as _, AsyncApp, Context, ElementId, Entity,
    InteractiveElement as _, IntoElement, ObjectFit, ParentElement as _, Render, Styled as _,
    StyledImage as _, WeakEntity, Window,
};

use crate::riot::{self, shop::AccountStore};

#[allow(dead_code)]
pub struct ShopDialogView {
    label: String,
    loading: bool,
    store: Option<AccountStore>,
    error: Option<String>,
}

impl ShopDialogView {
    async fn fetch_flow(label: String, this: WeakEntity<Self>, cx: &mut AsyncApp) {
        let yaml_path = crate::accounts::ACCOUNTS_DIR
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

        let yaml_path_clone = yaml_path.clone();
        let tokens = match cx
            .background_executor()
            .spawn(async move { riot::api::extract_session_tokens(&yaml_path_clone) })
            .await
        {
            Ok(t) => t,
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
        };

        let tokens_clone = tokens.clone();
        let access_token = match riot::spawn_tokio(async move {
            riot::api::get_access_token(&tokens_clone).await
        })
        .await
        {
            Ok(t) => t,
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
        };

        let at_clone = access_token.clone();
        let entitlements = match riot::spawn_tokio(async move {
            riot::api::get_entitlements_token(&at_clone).await
        })
        .await
        {
            Ok(t) => t,
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
        };

        let at_clone = access_token.clone();
        let (game_id, puuid) = match riot::spawn_tokio(async move {
            riot::api::get_player_info(&at_clone).await
        })
        .await
        {
            Ok(info) => info,
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
        };

        let client_version = match riot::spawn_tokio(async move {
            riot::api::cached_client_version().await
        })
        .await
        {
            Ok(v) => v,
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
        };

        let at_clone = access_token.clone();
        let ent_clone = entitlements.clone();
        let puuid_clone = puuid.clone();
        let cv_clone = client_version.clone();
        let store_data = match riot::spawn_tokio(async move {
            riot::api::get_storefront(&at_clone, &ent_clone, &puuid_clone, &cv_clone, "ap").await
        })
        .await
        {
            Ok(d) => d,
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
        };

        let at_clone = access_token.clone();
        let ent_clone = entitlements.clone();
        let puuid_clone = puuid.clone();
        let cv_clone = client_version.clone();
        let (vp, rp) = match riot::spawn_tokio(async move {
            riot::api::get_wallet(&at_clone, &ent_clone, &puuid_clone, &cv_clone, "ap").await
        })
        .await
        {
            Ok(v) => v,
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
        };

        let offers = store_data["SkinsPanelLayout"]["SingleItemStoreOffers"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let reset_secs = store_data["SkinsPanelLayout"]
            ["SingleItemOffersRemainingDurationInSeconds"]
            .as_u64()
            .unwrap_or(0);

        let offers_clone = offers.clone();
        let skins = match riot::spawn_tokio(async move {
            let mut tasks = Vec::new();
            for offer in offers_clone {
                let uuid = offer["OfferID"].as_str().unwrap_or("").to_string();
                let cost = offer["Cost"]["85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741"]
                    .as_u64()
                    .unwrap_or(0) as u32;

                tasks.push(async move {
                    let (name, icon_url) = riot::shop::get_skin_info(&uuid).await;
                    let icon_path = riot::shop::download_single_image(&icon_url).await;
                    riot::shop::SkinOffer {
                        name,
                        cost,
                        icon_url,
                        icon_path,
                    }
                });
            }
            Ok(futures_util::future::join_all(tasks).await)
        })
        .await
        {
            Ok(s) => s,
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    this.error = Some(e.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
        };

        // Completed!
        this.update(cx, |this, cx| {
            this.loading = false;
            this.store = Some(AccountStore {
                game_id,
                vp,
                rp,
                skins,
                reset_secs,
            });
            cx.notify();
        })
        .ok();
    }

    pub fn new(label: String, cx: &mut Context<Self>) -> Self {
        let label_clone = label.clone();
        cx.spawn(async move |this, cx| {
            Self::fetch_flow(label_clone, this, cx).await;
        })
        .detach();

        Self {
            label,
            loading: true,
            store: None,
            error: None,
        }
    }

    #[allow(dead_code)]
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        self.loading = true;
        self.error = None;
        self.store = None;
        cx.notify();

        let label_clone = self.label.clone();
        cx.spawn(async move |this, cx| {
            Self::fetch_flow(label_clone, this, cx).await;
        })
        .detach();
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

    fn render_skin_card(&self, skin: &riot::shop::SkinOffer, index: usize, theme: &gpui_kit::component::Theme) -> impl IntoElement {
        let skin_name = skin.name.clone();
        let youtube_query = format!("Valorant {}", skin_name).replace(' ', "+");
        let youtube_url = format!(
            "https://www.youtube.com/results?search_query={}",
            youtube_query
        );

        div()
            .id(ElementId::NamedInteger("skin_card".into(), index as u64))
            .flex()
            .flex_col()
            .flex_1()
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .bg(theme.background)
            .overflow_hidden()
            .hover(|s| s.border_color(theme.primary))
            .child(
                Button::new(ElementId::NamedInteger("skin_img_btn".into(), index as u64))
                    .ghost()
                    .h(px(100.0))
                    .w_full()
                    .p_2()
                    .rounded_none()
                    .bg(rgb(0x141e2b))
                    .tooltip("Click to search skin on YouTube")
                    .on_click(move |_, _, _| {
                        let _ = open::that(&youtube_url);
                    })
                    .child(if let Some(path) = &skin.icon_path {
                        img(path.clone())
                            .h_full()
                            .w_full()
                            .object_fit(ObjectFit::Contain)
                            .into_any_element()
                    } else {
                        Icon::new(gpui_kit::assets::IconName::Image)
                            .text_color(theme.muted_foreground)
                            .into_any_element()
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .line_clamp(1)
                            .child(skin.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_bold()
                            .text_color(theme.foreground)
                            .child(format!("{} VP", skin.cost)),
                    ),
            )
    }

    fn render_skeleton_card(&self, index: usize, theme: &gpui_kit::component::Theme) -> impl IntoElement {
        div()
            .id(ElementId::NamedInteger("skeleton_card".into(), index as u64))
            .flex()
            .flex_col()
            .flex_1()
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .bg(theme.background)
            .overflow_hidden()
            .child(
                Skeleton::new()
                    .h(px(100.0))
                    .w_full()
                    .rounded_none()
                    .bg(theme.secondary.opacity(0.3)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .gap_2()
                    .child(
                        Skeleton::new()
                            .h(px(14.0))
                            .w(px(110.0))
                            .rounded_sm(),
                    )
                    .child(
                        Skeleton::new()
                            .h(px(14.0))
                            .w(px(48.0))
                            .rounded_sm(),
                    ),
            )
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
                                Skeleton::new()
                                    .h(px(24.0))
                                    .w(px(72.0))
                                    .rounded_full(),
                            )
                            .child(
                                Skeleton::new()
                                    .h(px(24.0))
                                    .w(px(72.0))
                                    .rounded_full(),
                            )
                            .child(
                                Skeleton::new()
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
                                    .child(self.render_skeleton_card(0, theme))
                                    .child(self.render_skeleton_card(1, theme)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(self.render_skeleton_card(2, theme))
                                    .child(self.render_skeleton_card(3, theme)),
                            ),
                    )
                    .into_any_element()
            } else if let Some(err) = &self.error {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        Alert::error("shop_error", err.clone())
                            .title("Failed to load shop"),
                    )
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
                                Tag::color(ColorName::Neutral)
                                    .rounded_full()
                                    .gap_1p5()
                                    .child(Icon::new(gpui_kit::assets::IconName::Coins).xsmall())
                                    .child(format!("{} VP", store.vp)),
                            )
                            // RP Chip
                            .child(
                                Tag::color(ColorName::Neutral)
                                    .rounded_full()
                                    .gap_1p5()
                                    .child(Icon::new(gpui_kit::assets::IconName::Coins).xsmall())
                                    .child(format!("{} RP", store.rp)),
                            )
                            // Resets in Chip
                            .child(
                                Tag::color(ColorName::Neutral)
                                    .rounded_full()
                                    .gap_1p5()
                                    .child(Icon::new(gpui_kit::assets::IconName::Clock).xsmall())
                                    .child(format!(
                                        "Resets in {}",
                                        riot::shop::format_reset_time(store.reset_secs)
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
                                    .when_some(s1, |this, s| this.child(self.render_skin_card(s, 0, theme)))
                                    .when_some(s2, |this, s| this.child(self.render_skin_card(s, 1, theme))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .when_some(s3, |this, s| this.child(self.render_skin_card(s, 2, theme)))
                                    .when_some(s4, |this, s| this.child(self.render_skin_card(s, 3, theme))),
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
