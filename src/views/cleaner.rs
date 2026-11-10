use gpui_kit::component::WindowExt as _;
use gpui_kit::{App, Entity, Window};

use crate::app::AppView;

pub fn open_cleaner_dialog(
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
                "This will safely delete temporary shader caches from %LOCALAPPDATA%\\VALORANT to free up disk space. Your saved accounts and control settings will be kept, but in-game video and graphics settings will reset."
            )
            .confirm()
            .ok_text("Clean Cache")
            .on_ok(move |_, _, cx| {
                entity.update(cx, |app, cx| {
                    app.execute_clean_cache(cx);
                });
                true
            })
    });
}
