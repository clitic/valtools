use crate::riot::{AccessoryOffer, AccessoryType, SkinOffer};
use gpui_kit::{
    assets::IconName,
    base::StyledExt,
    component::{
        button::{Button, ButtonVariants},
        skeleton::Skeleton,
        tag::Tag,
        tooltip::Tooltip,
        Icon, Sizable, Theme,
    },
    div, img, px, rgb, ElementId, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    StatefulInteractiveElement, Styled, StyledImage,
};

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
        .h(px(150.0))
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
                .tooltip(skin.name.clone())
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
                    Icon::new(IconName::Image)
                        .text_color(theme.muted_foreground)
                        .into_any_element()
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .h(px(48.0))
                .p_2()
                .gap_2()
                .child(
                    div()
                        .id(ElementId::NamedInteger(
                            "skin_name_chip".into(),
                            index as u64,
                        ))
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .justify_center()
                        .tooltip({
                            let full_name = skin.name.clone();
                            move |window, cx| Tooltip::new(full_name.clone()).build(window, cx)
                        })
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.foreground)
                                .truncate()
                                .child(skin.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .truncate()
                                .child(tier.display_name()),
                        ),
                )
                .child(
                    div()
                        .id(ElementId::NamedInteger(
                            "skin_cost_chip".into(),
                            index as u64,
                        ))
                        .flex_shrink_0()
                        .tooltip(move |window, cx| {
                            Tooltip::new(tier.display_name()).build(window, cx)
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

pub fn render_accessory_card(
    accessory: &AccessoryOffer,
    index: usize,
    theme: &Theme,
) -> impl IntoElement {
    let item_name = accessory.name.clone();
    let youtube_query = format!("Valorant {}", item_name).replace(' ', "+");
    let youtube_url = format!(
        "https://www.youtube.com/results?search_query={}",
        youtube_query
    );
    let kc_color = rgb(0x40c4ff);

    div()
        .id(ElementId::NamedInteger(
            "accessory_card".into(),
            index as u64,
        ))
        .flex()
        .flex_col()
        .flex_1()
        .h(px(150.0))
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .overflow_hidden()
        .hover(move |s| s.border_color(kc_color.opacity(0.8)))
        .child(
            Button::new(ElementId::NamedInteger("acc_img_btn".into(), index as u64))
                .ghost()
                .h(px(100.0))
                .w_full()
                .p_2()
                .rounded_none()
                .bg(rgb(0x121c27))
                .tooltip(accessory.name.clone())
                .on_click(move |_, _, _| {
                    let _ = open::that(&youtube_url);
                })
                .child(if let Some(path) = &accessory.icon_path {
                    img(path.clone())
                        .h_full()
                        .w_full()
                        .object_fit(ObjectFit::Contain)
                        .into_any_element()
                } else if accessory.item_type == AccessoryType::Title {
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size_full()
                        .p_2()
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .text_color(kc_color)
                                .child(format!("“{}”", accessory.name)),
                        )
                        .into_any_element()
                } else {
                    Icon::new(IconName::Image)
                        .text_color(theme.muted_foreground)
                        .into_any_element()
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .h(px(48.0))
                .p_2()
                .gap_2()
                .child(
                    div()
                        .id(ElementId::NamedInteger(
                            "acc_name_chip".into(),
                            index as u64,
                        ))
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .justify_center()
                        .tooltip({
                            let full_name = accessory.name.clone();
                            move |window, cx| Tooltip::new(full_name.clone()).build(window, cx)
                        })
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.foreground)
                                .truncate()
                                .child(accessory.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .truncate()
                                .child(accessory.item_type.display_name()),
                        ),
                )
                .child(
                    div().flex_shrink_0().child(
                        Tag::custom(
                            kc_color.opacity(0.18).into(),
                            kc_color.into(),
                            kc_color.opacity(0.45).into(),
                        )
                        .small()
                        .rounded_full()
                        .gap_1()
                        .child(Icon::new(IconName::Coins).xsmall())
                        .child(format!("{} KC", accessory.cost)),
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
        .h(px(150.0))
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
                .child(Skeleton::new().h(px(42.0)).w(px(96.0)).rounded_md()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .h(px(48.0))
                .p_2()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .justify_center()
                        .gap_1()
                        .child(Skeleton::new().h(px(13.0)).w_full().rounded_sm())
                        .child(Skeleton::new().h(px(11.0)).w(px(48.0)).rounded_sm()),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .child(Skeleton::new().h(px(20.0)).w(px(72.0)).rounded_full()),
                ),
        )
}
