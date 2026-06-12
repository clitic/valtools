use slint::ComponentHandle;

use crate::AppWindow;

pub(crate) fn setup(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_minimize_window(move || {
        let ui = ui_handle.unwrap();
        ui.window().set_minimized(true);
    });

    let ui_handle = ui.as_weak();
    ui.on_close_window(move || {
        let ui = ui_handle.unwrap();
        let _ = ui.hide();
    });

    let ui_handle = ui.as_weak();
    ui.on_move_window(move |dx, dy| {
        let ui = ui_handle.unwrap();
        let scale = ui.window().scale_factor();
        let pos = ui.window().position();
        ui.window().set_position(slint::PhysicalPosition::new(
            pos.x + (dx * scale) as i32,
            pos.y + (dy * scale) as i32,
        ));
    });

    let ui_handle = ui.as_weak();
    ui.on_resize_window(move |dw, dh| {
        let ui = ui_handle.unwrap();
        let scale = ui.window().scale_factor();
        let size = ui.window().size();
        let new_w = (size.width as f32 + dw * scale).max(500.0 * scale) as u32;
        let new_h = (size.height as f32 + dh * scale).max(500.0 * scale) as u32;
        ui.window().set_size(slint::PhysicalSize::new(new_w, new_h));
    });

    ui.on_open_url(move |url| {
        let url_str = url.to_string();
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &url_str])
            .spawn();
    });
}
