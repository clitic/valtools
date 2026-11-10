use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    Icon, IconName, Sizable as _, Theme,
};
use gpui_kit::{div, px, Entity, IntoElement, ParentElement as _, Styled as _};

use super::cleaner::open_cleaner_dialog;
use crate::app::AppView;

pub fn render_footer(
    entity: Entity<AppView>,
    status_message: &str,
    is_error: bool,
    cache_size_str: &str,
    theme: &Theme,
) -> impl IntoElement {
    let size_str = cache_size_str.to_string();
    let status_str = status_message.to_string();

    div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(36.0))
        .px_4()
        .border_t_1()
        .border_color(theme.border)
        .bg(theme.secondary.opacity(0.3))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .min_w_0()
                .child(
                    Icon::new(if is_error {
                        IconName::TriangleAlert
                    } else {
                        IconName::Check
                    })
                    .xsmall()
                    .text_color(if is_error {
                        theme.danger
                    } else {
                        theme.primary
                    }),
                )
                .child(
                    div()
                        .text_xs()
                        .truncate()
                        .text_color(if is_error {
                            theme.danger
                        } else {
                            theme.muted_foreground
                        })
                        .child(status_str),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .flex_shrink_0()
                .child(
                    Button::new("btn_clean_cache")
                        .ghost()
                        .small()
                        .icon(Icon::new(gpui_kit::assets::IconName::BrushCleaning))
                        .tooltip(format!("Clean Cache ({})", size_str))
                        .on_click(move |_, window, cx| {
                            open_cleaner_dialog(size_str.clone(), entity.clone(), window, cx);
                        }),
                )
                .child(
                    Button::new("btn_github")
                        .icon(Icon::empty().path("icons/github.svg"))
                        .ghost()
                        .small()
                        .tooltip("Open GitHub source repository")
                        .on_click(|_, _, _| {
                            let _ = open::that("https://github.com/clitic/valtools");
                        }),
                ),
        )
}
