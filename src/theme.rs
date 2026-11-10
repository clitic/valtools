use gpui_kit::{
    component::{Theme, ThemeMode},
    px, rgb, App,
};

pub fn apply_theme(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);

    Theme::update(cx, |theme| {
        theme.colors.background = rgb(0x0f1923).into();
        theme.colors.foreground = rgb(0xece8e1).into();
        theme.colors.title_bar = rgb(0x0b1118).into();
        theme.colors.title_bar_border = rgb(0x1e2d3b).into();
        theme.colors.status_bar = rgb(0x0b1118).into();
        theme.colors.status_bar_border = rgb(0x1e2d3b).into();
        theme.colors.border = rgb(0x2a3d50).into();

        // Surface & Secondary backgrounds
        theme.colors.secondary = rgb(0x182430).into();
        theme.colors.secondary_foreground = rgb(0xa2b4c6).into();
        theme.colors.secondary_hover = rgb(0x223242).into();
        theme.colors.secondary_active = rgb(0x131c26).into();

        // Accent & Muted typography
        theme.colors.accent = rgb(0x223446).into();
        theme.colors.accent_foreground = rgb(0xffffff).into();
        theme.colors.muted = rgb(0x15202b).into();
        theme.colors.muted_foreground = rgb(0x76899c).into();

        // Primary Valorant Radiant Red
        theme.colors.primary = rgb(0xfa4454).into();
        theme.colors.primary_foreground = rgb(0xffffff).into();
        theme.colors.primary_hover = rgb(0xff5c6c).into();
        theme.colors.primary_active = rgb(0xe03b4a).into();

        // Primary Button
        theme.colors.button_primary = rgb(0xfa4454).into();
        theme.colors.button_primary_foreground = rgb(0xffffff).into();
        theme.colors.button_primary_hover = rgb(0xff5c6c).into();
        theme.colors.button_primary_active = rgb(0xe03b4a).into();

        // Secondary Button (Tactical Slate Blue)
        theme.colors.button_secondary = rgb(0x223344).into();
        theme.colors.button_secondary_foreground = rgb(0xffffff).into();
        theme.colors.button_secondary_hover = rgb(0x2d4358).into();
        theme.colors.button_secondary_active = rgb(0x182532).into();

        // Default Button & Input
        theme.colors.button = rgb(0x223344).into();
        theme.colors.button_foreground = rgb(0xffffff).into();
        theme.colors.button_hover = rgb(0x2d4358).into();
        theme.colors.button_active = rgb(0x182532).into();
        theme.colors.input = rgb(0x354a5f).into();

        // Skeleton placeholder loading (tactical slate pulse)
        theme.colors.skeleton = rgb(0x223446).into();

        // Danger Button
        theme.colors.danger = rgb(0xfa4454).into();
        theme.colors.danger_foreground = rgb(0xffffff).into();
        theme.colors.danger_hover = rgb(0xff5c6c).into();
        theme.colors.danger_active = rgb(0xe03b4a).into();
        theme.colors.button_danger = rgb(0xfa4454).into();
        theme.colors.button_danger_foreground = rgb(0xffffff).into();
        theme.colors.button_danger_hover = rgb(0xff5c6c).into();
        theme.colors.button_danger_active = rgb(0xe03b4a).into();

        // Status colors
        theme.colors.success = rgb(0x00f59b).into();
        theme.colors.success_foreground = rgb(0x0f1923).into();
        theme.colors.warning = rgb(0xffd700).into();
        theme.colors.warning_foreground = rgb(0x0f1923).into();

        // Popover & Lists
        theme.colors.popover = rgb(0x182430).into();
        theme.colors.popover_foreground = rgb(0xece8e1).into();
        theme.colors.list = rgb(0x0f1923).into();
        theme.colors.list_hover = rgb(0x182430).into();
        theme.colors.list_active = rgb(0x223446).into();

        // Tactical sharp radius
        theme.radius = px(4.0);
    });
}
