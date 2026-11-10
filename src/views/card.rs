use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    skeleton::Skeleton,
    tag::Tag,
    Icon, Sizable as _, Theme,
};
use gpui_kit::{
    div, img, px, rgb, ElementId, InteractiveElement as _, IntoElement, ObjectFit,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, StyledImage as _,
};

use crate::riot::SkinOffer;

pub fn render_skin_card(skin: &SkinOffer, index: usize, theme: &Theme) -> impl IntoElement {
    let skin_name = skin.name.clone();
    let youtube_query = format!("Valorant {}", skin_name).replace(' ', "+");
    let youtube_url = format!(
        "https://www.youtube.com/results?search_query={}",
        youtube_query
    );
    let tier = skin.tier;
    let tier_color = rgb(tier.color_hex());

    div()
        .id(ElementId::NamedInteger("skin_card".into(), index as u64))
        .flex()
        .flex_col()
        .flex_1()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .overflow_hidden()
        .hover(move |s| s.border_color(tier_color.opacity(0.8)))
        .child(
            Button::new(ElementId::NamedInteger("skin_img_btn".into(), index as u64))
                .ghost()
                .h(px(100.0))
                .w_full()
                .p_2()
                .rounded_none()
                .bg(rgb(0x121c27))
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
                        .min_w_0()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .truncate()
                        .child(skin.name.clone()),
                )
                .child(
                    div()
                        .id(ElementId::NamedInteger(
                            "skin_cost_chip".into(),
                            index as u64,
                        ))
                        .tooltip(move |window, cx| {
                            gpui_kit::component::tooltip::Tooltip::new(tier.display_name())
                                .build(window, cx)
                        })
                        .child(
                            Tag::custom(
                                tier_color.opacity(0.18).into(),
                                tier_color.into(),
                                tier_color.opacity(0.45).into(),
                            )
                            .small()
                            .rounded_full()
                            .gap_1()
                            .child(
                                img(tier.icon_asset())
                                    .size(px(14.0))
                                    .object_fit(ObjectFit::Contain),
                            )
                            .child(format!("{} VP", skin.cost)),
                        ),
                ),
        )
}

pub fn render_skeleton_card(index: usize, theme: &Theme) -> impl IntoElement {
    div()
        .id(ElementId::NamedInteger(
            "skeleton_card".into(),
            index as u64,
        ))
        .flex()
        .flex_col()
        .flex_1()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .overflow_hidden()
        .child(
            div()
                .h(px(100.0))
                .w_full()
                .p_2()
                .bg(rgb(0x121c27))
                .flex()
                .items_center()
                .justify_center()
                .child(Skeleton::new().h(px(54.0)).w_full().rounded_md()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p_2()
                .gap_2()
                .child(Skeleton::new().h(px(14.0)).w(px(110.0)).rounded_sm())
                .child(Skeleton::new().h(px(20.0)).w(px(64.0)).rounded_full()),
        )
}
