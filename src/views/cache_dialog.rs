use gpui_kit::component::{button::ButtonVariant, WindowExt as _};
use gpui_kit::{App, Entity, Window};

use crate::app::AppView;

pub fn open_cache_dialog(
    size_str: String,
    entity: Entity<AppView>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, _| {
        let entity = entity.clone();
        alert
            .title(format!("Clean {} of Cache?", size_str))
            .description(
                "This will permanently delete temporary shader caches, crash logs, and web data from %LOCALAPPDATA%\\VALORANT.\n\nYour game installation, saved accounts, and game settings are NOT deleted.\n\nThis action is irreversible."
            )
            .confirm()
            .ok_text("Clean Cache")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                entity.update(cx, |app, cx| {
                    app.execute_clean_cache(cx);
                });
                true
            })
    });
}
