use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariant, ButtonVariants},
    Icon, Sizable as _, Theme, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, rgb, svg, ElementId, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Styled as _,
};

use crate::{accounts::Account, app::AppView};

pub fn render_account_row(
    account: &Account,
    index: usize,
    entity: Entity<AppView>,
    theme: &Theme,
) -> impl IntoElement {
    let label = account.label.clone();
    let is_active = account.is_active;

    let label_for_shop = label.clone();
    let label_for_riot = label.clone();
    let label_for_val = label.clone();
    let label_for_del = label.clone();
    let entity_for_riot = entity.clone();
    let entity_for_val = entity.clone();
    let entity_for_del = entity;

    div()
        .id(ElementId::NamedInteger("account_row".into(), index as u64))
        .flex()
        .items_center()
        .justify_between()
        .p_3()
        .rounded_lg()
        .border_1()
        .border_color(if is_active {
            theme.primary.opacity(0.4)
        } else {
            theme.border
        })
        .bg(if is_active {
            theme.primary.opacity(0.05)
        } else {
            theme.secondary.opacity(0.25)
        })
        .hover(|s| s.bg(theme.accent.opacity(0.2)))
        // Left section: Label + Active Badge
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .font_medium()
                        .text_sm()
                        .text_color(if is_active {
                            theme.foreground
                        } else {
                            theme.foreground.opacity(0.85)
                        })
                        .child(label.clone()),
                )
                .when(is_active, |this| {
                    this.child(
                        Badge::new()
                            .child("active"),
                    )
                }),
        )
        // Right section: Action buttons
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                // Shop Button
                .child(
                    Button::new(ElementId::NamedInteger("btn_shop".into(), index as u64))
                        .outline()
                        .small()
                        .icon(Icon::new(gpui_kit::assets::IconName::Store))
                        .label("Shop")
                        .tooltip("View Daily Shop")
                        .on_click(move |_, window, cx| {
                            AppView::open_shop_dialog(label_for_shop.clone(), window, cx);
                        }),
                )
                // Riot Client Button
                .child(
                    Button::new(ElementId::NamedInteger("btn_riot".into(), index as u64))
                        .outline()
                        .small()
                        .tooltip("Launch Riot Client")
                        .child(
                            svg()
                                .path("icons/riot.svg")
                                .size(px(14.0))
                                .text_color(rgb(0xd13639)),
                        )
                        .on_click(move |_, _, cx| {
                            let label = label_for_riot.clone();
                            entity_for_riot.update(cx, |app, cx| {
                                app.switch_account(label, cx);
                            });
                        }),
                )
                // Valorant Button (Primary)
                .child(
                    Button::new(ElementId::NamedInteger("btn_val".into(), index as u64))
                        .primary()
                        .small()
                        .tooltip("Launch Valorant")
                        .child(
                            svg()
                                .path("icons/valorant.svg")
                                .size(px(14.0))
                                .text_color(rgb(0xffffff)),
                        )
                        .on_click(move |_, _, cx| {
                            let label = label_for_val.clone();
                            entity_for_val.update(cx, |app, cx| {
                                app.launch_valorant(label, cx);
                            });
                        }),
                )
                // Delete Button (Danger)
                .child(
                    Button::new(ElementId::NamedInteger("btn_del".into(), index as u64))
                        .ghost()
                        .danger()
                        .small()
                        .icon(Icon::new(gpui_kit::assets::IconName::Trash))
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
                                    .ok_variant(ButtonVariant::Danger)
                                    .on_ok(move |_, _, cx| {
                                        entity.update(cx, |app, cx| {
                                            app.delete_account(&label_owned, cx);
                                        });
                                        true
                                    })
                            });
                        }),
                ),
        )
}
