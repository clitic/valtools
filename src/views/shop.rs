use crate::{
    riot::{store, AccountStore},
    views::card::{render_accessory_card, render_skeleton_card, render_skin_card},
};
use gpui_kit::{
    assets::IconName,
    base::StyledExt,
    component::{
        alert::Alert, skeleton::Skeleton, tag::Tag, ActiveTheme, Icon, Sizable, WindowExt,
    },
    div, px, App, AppContext, AsyncApp, Context, Entity, IntoElement, ParentElement, Render,
    Styled, WeakEntity, Window,
};

pub struct ShopDialogView {
    loading: bool,
    store: Option<AccountStore>,
    error: Option<String>,
}

impl ShopDialogView {
    async fn fetch_flow(label: String, this: WeakEntity<Self>, cx: &mut AsyncApp) {
        let yaml_path = crate::session::account_path(&label);

        if !yaml_path.exists() {
            this.update(cx, |this, cx| {
                this.loading = false;
                this.error = Some(format!("No saved session found for {}", label));
                cx.notify();
            })
            .ok();
            return;
        }

        let store_res = cx
            .background_executor()
            .spawn(async move { store::fetch_store(&yaml_path) })
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
                    this.error = Some(e);
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
        let title = format!("{}'s Store", label);
        let shop_view: Entity<Self> = cx.new(|cx| Self::new(label, cx));

        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title(title.clone())
                .width(px(720.0))
                .child(shop_view.clone())
        });
    }
}

fn format_duration(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        if hours > 0 {
            format!("{}d {}h left", days, hours)
        } else {
            format!("{}d left", days)
        }
    } else if hours > 0 {
        format!("{}h {}m left", hours, mins)
    } else if mins > 0 {
        format!("{}m left", mins)
    } else {
        "< 1m left".to_string()
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
                    .child(
                        // Section 1 header skeleton
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(Skeleton::new().size(px(12.0)).rounded_sm())
                                    .child(Skeleton::new().h(px(14.0)).w(px(72.0)).rounded_sm()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(Skeleton::new().h(px(20.0)).w(px(68.0)).rounded_full())
                                    .child(Skeleton::new().h(px(20.0)).w(px(60.0)).rounded_full())
                                    .child(Skeleton::new().h(px(20.0)).w(px(94.0)).rounded_full()),
                            ),
                    )
                    .child(
                        // 4 cards in one line skeleton
                        div()
                            .flex()
                            .gap_2()
                            .child(render_skeleton_card(0, theme))
                            .child(render_skeleton_card(1, theme))
                            .child(render_skeleton_card(2, theme))
                            .child(render_skeleton_card(3, theme)),
                    )
                    .child(
                        // Section 2 header skeleton
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pt_2()
                            .border_t_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(Skeleton::new().size(px(12.0)).rounded_sm())
                                    .child(Skeleton::new().h(px(14.0)).w(px(106.0)).rounded_sm()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(Skeleton::new().h(px(20.0)).w(px(68.0)).rounded_full())
                                    .child(Skeleton::new().h(px(20.0)).w(px(88.0)).rounded_full()),
                            ),
                    )
                    .child(
                        // 4 cards in second line skeleton
                        div()
                            .flex()
                            .gap_2()
                            .child(render_skeleton_card(4, theme))
                            .child(render_skeleton_card(5, theme))
                            .child(render_skeleton_card(6, theme))
                            .child(render_skeleton_card(7, theme)),
                    )
                    .into_any_element()
            } else if let Some(err) = &self.error {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(Alert::error("shop_error", err.clone()).title("Failed to load store"))
                    .into_any_element()
            } else if let Some(store) = &self.store {
                let daily_reset = format_duration(store.reset_secs);
                let acc_reset = format_duration(store.accessory_reset_secs);

                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    // Line 1: Daily Offers (4 cards in one line)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(Icon::new(IconName::ShoppingCart).xsmall())
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(theme.foreground)
                                            .child("Daily Offers"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(
                                        Tag::new()
                                            .small()
                                            .rounded_full()
                                            .gap_1()
                                            .child(Icon::new(IconName::Coins).xsmall())
                                            .child(format!("{} VP", store.vp)),
                                    )
                                    .child(
                                        Tag::new()
                                            .small()
                                            .rounded_full()
                                            .gap_1()
                                            .child(Icon::new(IconName::Coins).xsmall())
                                            .child(format!("{} RP", store.rp)),
                                    )
                                    .child(
                                        Tag::new()
                                            .small()
                                            .rounded_full()
                                            .gap_1()
                                            .child(Icon::new(IconName::Clock).xsmall())
                                            .child(daily_reset),
                                    ),
                            ),
                    )
                    .child(if store.skins.is_empty() {
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .h(px(80.0))
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("No daily skin offers available")
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .gap_2()
                            .children(
                                store
                                    .skins
                                    .iter()
                                    .enumerate()
                                    .map(|(col_idx, skin)| render_skin_card(skin, col_idx, theme)),
                            )
                            .children(
                                (0..(4usize.saturating_sub(store.skins.len())))
                                    .map(|_| div().flex_1()),
                            )
                            .into_any_element()
                    })
                    // Line 2: Accessories Store (4 cards in another line)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pt_2()
                            .border_t_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(Icon::new(IconName::Coins).xsmall())
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(theme.foreground)
                                            .child("Accessories Store"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(
                                        Tag::new()
                                            .small()
                                            .rounded_full()
                                            .gap_1()
                                            .child(Icon::new(IconName::Coins).xsmall())
                                            .child(format!("{} KC", store.kc)),
                                    )
                                    .child(
                                        Tag::new()
                                            .small()
                                            .rounded_full()
                                            .gap_1()
                                            .child(Icon::new(IconName::Clock).xsmall())
                                            .child(acc_reset),
                                    ),
                            ),
                    )
                    .child(if store.accessories.is_empty() {
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .h(px(80.0))
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("No accessories offers available")
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .gap_2()
                            .children(
                                store.accessories.iter().enumerate().map(|(col_idx, acc)| {
                                    render_accessory_card(acc, col_idx, theme)
                                }),
                            )
                            .children(
                                (0..(4usize.saturating_sub(store.accessories.len())))
                                    .map(|_| div().flex_1()),
                            )
                            .into_any_element()
                    })
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
