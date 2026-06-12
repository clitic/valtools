#![windows_subsystem = "windows"]

mod accounts;
mod cache;
mod client;
mod error;
mod handlers;
mod riot;

use slint::{ModelRc, SharedString, VecModel};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::LazyLock};

slint::include_modules!();

static LOCAL_APP_DATA: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(
        std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".into()),
    )
});

fn accounts_to_model(accts: &[accounts::Account]) -> ModelRc<AccountData> {
    let items = accts
        .iter()
        .map(|a| AccountData {
            label: SharedString::from(&a.label),
            is_active: a.is_active,
        })
        .collect::<Vec<AccountData>>();
    ModelRc::new(Rc::new(VecModel::from(items)))
}

fn cache_entries_to_model(entries: &[cache::CacheEntry]) -> ModelRc<CacheEntryData> {
    let items = entries
        .iter()
        .map(|e| CacheEntryData {
            name: SharedString::from(&e.name),
            size_display: SharedString::from(cache::format_size(e.size)),
            size: e.size as i32,
            checked: true,
        })
        .collect::<Vec<CacheEntryData>>();
    ModelRc::new(Rc::new(VecModel::from(items)))
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // Initial data load
    let accts = accounts::list_accounts();
    ui.set_accounts(accounts_to_model(&accts));

    let entries = cache::list_cache_entries();
    ui.set_cache_entries(cache_entries_to_model(&entries));
    handlers::cache::update_cache_button(&ui);

    // Store entry paths for cleaning
    let cache_paths: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(
        entries.iter().map(|e| e.path.clone()).collect::<Vec<PathBuf>>(),
    ));

    // Register all UI callbacks
    handlers::setup_all(&ui, cache_paths);

    ui.run()
}
