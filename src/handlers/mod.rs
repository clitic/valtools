mod account;
pub(crate) mod cache;
mod shop;
mod window;

use slint::SharedString;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::{accounts, accounts_to_model, AppWindow};

pub(crate) fn set_status(ui: &AppWindow, msg: &str, is_error: bool) {
    ui.set_status_message(SharedString::from(msg));
    ui.set_status_is_error(is_error);
}

fn set_cache_msg(ui: &AppWindow, msg: &str, is_error: bool) {
    ui.set_cache_status(SharedString::from(msg));
    ui.set_cache_is_error(is_error);
}

fn refresh_accounts_model(ui: &AppWindow) {
    ui.set_accounts(accounts_to_model(&accounts::list_accounts()));
}

/// Register all UI callbacks on the given window.
pub(crate) fn setup_all(ui: &AppWindow, cache_paths: Rc<RefCell<Vec<PathBuf>>>) {
    window::setup(ui);
    account::setup(ui);
    cache::setup(ui, cache_paths);
    shop::setup(ui);
}
