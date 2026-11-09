#![windows_subsystem = "windows"]

mod accounts;
mod app;
mod assets;
mod cache;
mod client;
mod error;
mod riot;
mod views;

use gpui_kit::*;
use std::{path::PathBuf, sync::LazyLock};

pub static LOCAL_APP_DATA: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(
        std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".into()),
    )
});

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}\n", info);
        let _ = std::fs::write("valtools_panic.log", &msg);
        eprintln!("{}", msg);
    }));

    gpui_kit::application()
        .with_assets(assets::AppAssets)
        .run(|cx| {
            gpui_kit::init(cx);

            // Set Light theme
            gpui_kit::component::Theme::change(
                gpui_kit::component::ThemeMode::Light,
                None,
                cx,
            );

            let options = gpui_kit::WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(760.0), px(580.0)),
                    cx,
                ))),
                titlebar: Some(gpui_kit::component::TitleBar::title_bar_options()),
                ..Default::default()
            };

            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| app::AppView::new(window, cx))
            })
            .expect("Failed to open ValTools window");
        });
}
