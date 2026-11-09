use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    alert::Alert,
    button::{Button, ButtonVariants},
    dialog::{DialogClose, DialogFooter},
    spinner::Spinner,
    ActiveTheme as _, Icon, Sizable as _, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, img, px, rgb, App, AppContext as _, AsyncApp, Context, ElementId, Entity,
    InteractiveElement as _, IntoElement, ObjectFit, ParentElement as _, Render, Styled as _,
    StyledImage as _, WeakEntity, Window,
};
use std::process::Command;

use crate::riot::{self, shop::AccountStore};

#[allow(dead_code)]
pub struct ShopDialogView {
    label: String,
    loading: bool,
    status_msg: String,
    store: Option<AccountStore>,
    error: Option<String>,
}

impl ShopDialogView {
    async fn fetch_flow(label: String, this: WeakEntity<Self>, cx: &mut AsyncApp) {
        // Step 1: Loading session
        this.update(cx, |this, cx| {
            this.status_msg = "Loading session...".into();
            cx.notify();
        })
        .ok();

        let yaml_path = crate::accounts::ACCOUNTS_DIR
            .join(&label)
            .join("RiotGamesPrivateSettings.yaml");

        if !yaml_path.exists() {
            this.update(cx, |this, cx| {
                this.loading = false;
                this.error = Some(format!("No saved session found for '{}'", label));
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

        // Step 2: Authenticating with Riot
        this.update(cx, |this, cx| {
            this.status_msg = "Authenticating with Riot...".into();
            cx.notify();
        })
        .ok();

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

        // Step 3: Fetching entitlements
        this.update(cx, |this, cx| {
            this.status_msg = "Fetching entitlements...".into();
            cx.notify();
        })
        .ok();

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

        // Step 4: Fetching player info
        this.update(cx, |this, cx| {
            this.status_msg = "Fetching player info...".into();
            cx.notify();
        })
        .ok();

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

        // Step 5: Checking client version
        this.update(cx, |this, cx| {
            this.status_msg = "Checking client version...".into();
            cx.notify();
        })
        .ok();

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

        // Step 6: Fetching storefront
        this.update(cx, |this, cx| {
            this.status_msg = "Fetching storefront...".into();
            cx.notify();
        })
        .ok();

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

        // Step 7: Fetching VP balance
        this.update(cx, |this, cx| {
            this.status_msg = "Fetching VP balance...".into();
            cx.notify();
        })
        .ok();

        let at_clone = access_token.clone();
        let ent_clone = entitlements.clone();
        let puuid_clone = puuid.clone();
        let cv_clone = client_version.clone();
        let vp = match riot::spawn_tokio(async move {
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

        // Step 8: Resolving skin details & downloading images concurrently
        this.update(cx, |this, cx| {
            this.status_msg = "Resolving skins & downloading images...".into();
            cx.notify();
        })
        .ok();

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
            status_msg: "Initializing...".into(),
            store: None,
            error: None,
        }
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        self.loading = true;
        self.status_msg = "Initializing...".into();
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
        let label_title = label.clone();
        let shop_view: Entity<Self> = cx.new(|cx| Self::new(label, cx));
        let shop_view_clone = shop_view.clone();
        let shop_view_for_reload = shop_view.clone();

        window.open_dialog(cx, move |dialog, _, _| {
            let shop_view_for_reload = shop_view_for_reload.clone();
            dialog
                .title(format!("Daily Shop - {}", label_title))
                .width(px(560.0))
                .child(shop_view_clone.clone())
                .footer(
                    DialogFooter::new()
                        .child(
                            Button::new("reload_shop")
                                .outline()
                                .icon(Icon::new(gpui_kit::assets::IconName::RotateCw))
                                .label("Reload Shop")
                                .on_click(move |_, _, cx| {
                                    shop_view_for_reload.update(cx, |this, cx| {
                                        this.reload(cx);
                                    });
                                }),
                        )
                        .child(DialogClose::new().child(Button::new("close_shop").outline().label("Close"))),
                )
        });
    }

    fn render_skin_card(&self, skin: &riot::shop::SkinOffer, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let skin_name = skin.name.clone();
        let youtube_query = format!("Valorant {}", skin_name).replace(' ', "+");
        let youtube_url = format!(
            "https://www.youtube.com/results?search_query={}",
            youtube_query
        );
        let youtube_url_clone = youtube_url.clone();

        div()
            .id(ElementId::NamedInteger("skin_card".into(), index as u64))
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(220.0))
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .overflow_hidden()
            .hover(|s| s.border_color(cx.theme().primary))
            .child(
                Button::new(ElementId::NamedInteger("skin_img_btn".into(), index as u64))
                    .ghost()
                    .h(px(100.0))
                    .w_full()
                    .p_2()
                    .rounded_none()
                    .bg(rgb(0x141e2b))
                    .tooltip("Click to search skin on YouTube")
                    .on_click(move |_, _, cx| {
                        cx.open_url(&youtube_url);
                        let _ = Command::new("cmd")
                            .args(["/C", "start", "", &youtube_url_clone])
                            .spawn();
                    })
                    .child(if let Some(path) = &skin.icon_path {
                        img(path.clone())
                            .h_full()
                            .w_full()
                            .object_fit(ObjectFit::Contain)
                            .into_any_element()
                    } else {
                        Icon::new(gpui_kit::assets::IconName::Image)
                            .text_color(cx.theme().muted_foreground)
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
                            .text_color(cx.theme().foreground)
                            .line_clamp(1)
                            .child(skin.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_bold()
                            .text_color(rgb(0xcca010))
                            .child(format!("{} VP", skin.cost)),
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
            .w(px(500.0))
            .gap_4()
            .p_2()
            .child(if self.loading {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .py_8()
                    .gap_3()
                    .child(Spinner::new())
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(self.status_msg.clone()),
                    )
                    .into_any_element()
            } else if let Some(err) = &self.error {
                let this_entity = cx.entity().clone();
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        Alert::error("shop_error", err.clone())
                            .title("Failed to load shop"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child("Please ensure the Riot session is still active and valid."),
                            )
                            .child(
                                Button::new("btn_retry_shop")
                                    .primary()
                                    .small()
                                    .icon(Icon::new(gpui_kit::assets::IconName::RotateCw))
                                    .label("Retry")
                                    .on_click(move |_, _, cx| {
                                        this_entity.update(cx, |this, cx| {
                                            this.reload(cx);
                                        });
                                    }),
                            ),
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
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_2()
                            .rounded_md()
                            .bg(theme.secondary.opacity(0.4))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .font_bold()
                                            .text_sm()
                                            .text_color(theme.foreground)
                                            .child(store.game_id.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                Icon::new(gpui_kit::assets::IconName::Sparkles)
                                                    .xsmall()
                                                    .text_color(rgb(0xcca010)),
                                            )
                                            .child(
                                                div()
                                                    .font_bold()
                                                    .text_xs()
                                                    .text_color(rgb(0xcca010))
                                                    .child(format!("{} VP", store.vp)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                Icon::new(gpui_kit::assets::IconName::Clock)
                                                    .xsmall()
                                                    .text_color(theme.muted_foreground),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.muted_foreground)
                                                    .child(riot::shop::format_reset_time(
                                                        store.reset_secs,
                                                    )),
                                            ),
                                    ),
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
                                    .when_some(s1, |this, s| this.child(self.render_skin_card(s, 0, cx)))
                                    .when_some(s2, |this, s| this.child(self.render_skin_card(s, 1, cx))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .when_some(s3, |this, s| this.child(self.render_skin_card(s, 2, cx)))
                                    .when_some(s4, |this, s| this.child(self.render_skin_card(s, 3, cx))),
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
