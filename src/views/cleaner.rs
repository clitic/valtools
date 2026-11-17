use crate::{app::AppView, globals::LOCAL_APP_DATA};
use gpui_kit::{
    assets::IconName,
    base::StyledExt,
    component::{button::ButtonVariant, tooltip::Tooltip, ActiveTheme, Icon, WindowExt},
    div, px, App, Entity, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled,
    Window,
};

pub fn open_cleaner_dialog(
    size_str: String,
    entity: Entity<AppView>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let theme = cx.theme();
        let entity = entity.clone();

        alert
            .width(px(460.0))
            .title(format!("Clean {} Cache?", size_str))
            .description(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_1()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Clears temporary shader caches, match replays, and logs from")
                    .child(
                        div()
                            .id("btn_open_valorant_dir")
                            .text_color(theme.muted_foreground)
                            .cursor_pointer()
                            .hover(|s| s.text_color(theme.foreground))
                            .tooltip(|window, cx| {
                                Tooltip::new("Click to open in Windows Explorer").build(window, cx)
                            })
                            .on_click(|_, _, _| {
                                let dir = LOCAL_APP_DATA.join("VALORANT");
                                if dir.exists() {
                                    let _ = open::that(&dir);
                                } else {
                                    let _ = open::that(&*LOCAL_APP_DATA);
                                }
                            })
                            .child(r"%LOCALAPPDATA%\VALORANT"),
                    )
                    .child("to free up disk space."),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .mt_1()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.secondary)
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(theme.border)
                            .bg(theme.background.opacity(0.4))
                            .child(
                                Icon::new(IconName::SlidersHorizontal)
                                    .size(px(12.0))
                                    .text_color(theme.primary),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(theme.foreground)
                                    .child("These are the in-game settings that will reset:"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(theme.border.opacity(0.4))
                            .text_xs()
                            .child(div().text_color(theme.muted_foreground).child("General"))
                            .child(
                                Icon::new(IconName::ArrowRight)
                                    .size(px(10.0))
                                    .text_color(theme.muted_foreground),
                            )
                            .child(div().text_color(theme.muted_foreground).child("Network"))
                            .child(
                                Icon::new(IconName::ArrowRight)
                                    .size(px(10.0))
                                    .text_color(theme.muted_foreground),
                            )
                            .child(
                                div()
                                    .font_medium()
                                    .text_color(theme.foreground)
                                    .child("Network Buffering"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(theme.border.opacity(0.4))
                            .text_xs()
                            .child(div().text_color(theme.muted_foreground).child("Video"))
                            .child(
                                Icon::new(IconName::ArrowRight)
                                    .size(px(10.0))
                                    .text_color(theme.muted_foreground),
                            )
                            .child(
                                div()
                                    .font_medium()
                                    .text_color(theme.foreground)
                                    .child("General"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_3()
                            .py_2()
                            .text_xs()
                            .child(div().text_color(theme.muted_foreground).child("Video"))
                            .child(
                                Icon::new(IconName::ArrowRight)
                                    .size(px(10.0))
                                    .text_color(theme.muted_foreground),
                            )
                            .child(
                                div()
                                    .font_medium()
                                    .text_color(theme.foreground)
                                    .child("Graphics Quality"),
                            ),
                    ),
            )
            .confirm()
            .ok_text("Clean Cache")
            .ok_variant(ButtonVariant::Danger)
            .cancel_text("Cancel")
            .on_ok(move |_, _, cx| {
                entity.update(cx, |app, cx| {
                    app.execute_clean_cache(cx);
                });
                true
            })
    });
}
