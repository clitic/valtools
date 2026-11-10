use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    Icon, IconName, Theme,
};
use gpui_kit::{div, Entity, IntoElement, ParentElement as _, Styled as _};

use crate::app::AppView;

pub fn render_header(entity: Entity<AppView>, theme: &Theme) -> impl IntoElement {
    let entity_save = entity.clone();
    let entity_add = entity;

    div()
        .flex()
        .items_center()
        .justify_between()
        .p_4()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.secondary.opacity(0.3))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .min_w_0()
                .child(
                    div()
                        .font_bold()
                        .text_base()
                        .text_color(theme.foreground)
                        .child("Account Switcher"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child("Switch Riot accounts without logging out"),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .flex_shrink_0()
                .child(
                    Button::new("btn_save_session")
                        .secondary()
                        .icon(Icon::new(gpui_kit::assets::IconName::Save))
                        .label("Save Session")
                        .tooltip("Save the current logged-in Riot session")
                        .on_click(move |_, window, cx| {
                            entity_save.update(cx, |this, cx| {
                                this.save_session(window, cx);
                            });
                        }),
                )
                .child(
                    Button::new("btn_add_account")
                        .primary()
                        .icon(IconName::Plus)
                        .label("Add Account")
                        .tooltip("Log in with a new account via Riot Client")
                        .on_click(move |_, window, cx| {
                            entity_add.update(cx, |this, cx| {
                                this.prepare_new_login(window, cx);
                            });
                        }),
                ),
        )
}
