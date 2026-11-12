use crate::{app::AppView, session::Account};
use gpui_kit::{
    assets::IconName,
    base::StyledExt,
    component::{
        button::{Button, ButtonVariant, ButtonVariants},
        Icon, Sizable, Theme, WindowExt,
    },
    div, px, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, Styled,
};

pub fn render_account_row(
    account: &Account,
    index: usize,
    entity: Entity<AppView>,
    theme: &Theme,
) -> impl IntoElement {
    let label = account.label.clone();
    let is_active = account.is_active;

    let label_for_del = label.clone();
    let label_for_shop = label.clone();
    let label_for_riot = label.clone();
    let entity_for_del = entity.clone();
    let entity_for_riot = entity;

    div()
        .id(ElementId::NamedInteger("account_row".into(), index as u64))
        .flex()
        .items_center()
        .justify_between()
        .p_3()
        .rounded_lg()
        .border_1()
        .border_color(if is_active {
            theme.primary.opacity(0.6)
        } else {
            theme.border
        })
        .bg(if is_active {
            theme.primary.opacity(0.12)
        } else {
            theme.secondary.opacity(0.4)
        })
        .hover(|s| s.bg(theme.accent.opacity(0.4)))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .min_w_0()
                .child(
                    div()
                        .font_medium()
                        .text_sm()
                        .truncate()
                        .text_color(if is_active {
                            theme.foreground
                        } else {
                            theme.foreground.opacity(0.85)
                        })
                        .child(label.clone()),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .flex_shrink_0()
                .child(
                    Button::new(ElementId::NamedInteger("btn_del".into(), index as u64))
                        .ghost()
                        .small()
                        .icon(Icon::new(IconName::Trash))
                        .tooltip("Delete Saved Session")
                        .on_click(move |_, window, cx| {
                            let entity = entity_for_del.clone();
                            let label = label_for_del.clone();
                            window.open_alert_dialog(cx, move |alert, _, _| {
                                let entity = entity.clone();
                                let label_owned = label.clone();
                                alert
                                    .title(format!("Delete \"{}\"?", label))
                                    .description(
                                        "This will permanently remove the saved session from disk. This action cannot be undone."
                                    )
                                    .confirm()
                                    .ok_text("Delete")
                                    .on_ok(move |_, _, cx| {
                                        entity.update(cx, |app, cx| {
                                            app.delete_account(&label_owned, cx);
                                        });
                                        true
                                    })
                            });
                        }),
                )
                .child(
                    Button::new(ElementId::NamedInteger("btn_shop".into(), index as u64))
                        .secondary()
                        .small()
                        .w(px(76.0))
                        .icon(Icon::new(IconName::ShoppingCart))
                        .label("Store")
                        .tooltip("View Daily Store")
                        .on_click(move |_, window, cx| {
                            AppView::open_shop_dialog(label_for_shop.clone(), window, cx);
                        }),
                )
                .child(
                    Button::new(ElementId::NamedInteger("btn_riot".into(), index as u64))
                        .primary()
                        .small()
                        .w(px(76.0))
                        .icon(Icon::default().path("icons/riot.svg"))
                        .label("Riot")
                        .tooltip("Launch Riot Client")
                        .on_click(move |_, window, cx| {
                            let entity = entity_for_riot.clone();
                            let label = label_for_riot.clone();
                            window.open_alert_dialog(cx, move |alert, _, _| {
                                let entity = entity.clone();
                                let label_owned = label.clone();
                                alert
                                    .title(format!("Launch Riot Client as \"{}\"?", label))
                                    .description(format!(
                                        "Do you want to switch accounts and launch Riot Client as \"{}\"?",
                                        label
                                    ))
                                    .confirm()
                                    .ok_text("Launch")
                                    .ok_variant(ButtonVariant::Primary)
                                    .on_ok(move |_, _, cx| {
                                        entity.update(cx, |app, cx| {
                                            app.switch_account(label_owned.clone(), cx);
                                        });
                                        true
                                    })
                            });
                        }),
                ),
        )
}
