use slint::{ComponentHandle, Model};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::{cache, cache_entries_to_model, AppWindow};
use super::set_cache_msg;

pub(crate) fn update_cache_button(ui: &AppWindow) {
    let model = ui.get_cache_entries();
    let selected_size: i32 = (0..model.row_count())
        .filter_map(|i| {
            let e = model.row_data(i)?;
            if e.checked {
                Some(e.size)
            } else {
                None
            }
        })
        .sum();
    ui.set_cache_size_display(slint::SharedString::from(format!(
        "{}",
        cache::format_size(selected_size as u64)
    )));
}

pub(crate) fn setup(ui: &AppWindow, cache_paths: Rc<RefCell<Vec<PathBuf>>>) {
    let ui_handle = ui.as_weak();
    ui.on_toggle_cache_entry(move |idx, checked| {
        let ui = ui_handle.unwrap();
        let model = ui.get_cache_entries();
        if let Some(mut entry) = model.row_data(idx as usize) {
            entry.checked = checked;
            model.set_row_data(idx as usize, entry);
        }
        update_cache_button(&ui);
    });

    let paths_ref = cache_paths.clone();
    let ui_handle = ui.as_weak();
    ui.on_clean_cache(move || {
        let ui = ui_handle.unwrap();
        let model = ui.get_cache_entries();
        let to_clean = {
            let paths = paths_ref.borrow();
            (0..model.row_count())
                .filter_map(|i| {
                    let entry = model.row_data(i)?;
                    if entry.checked {
                        Some(paths[i].clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<PathBuf>>()
        };

        if to_clean.is_empty() {
            set_cache_msg(&ui, "Select items to clean.", true);
            return;
        }

        let (deleted, freed, errors) = cache::clean_entries(&to_clean);
        if errors.is_empty() {
            set_cache_msg(
                &ui,
                &format!(
                    "Cleaned {} items, {} freed.",
                    deleted,
                    cache::format_size(freed)
                ),
                false,
            );
        } else {
            set_cache_msg(
                &ui,
                &format!(
                    "Cleaned {} items ({}). {} failed: {}",
                    deleted,
                    cache::format_size(freed),
                    errors.len(),
                    errors.first().unwrap_or(&String::new())
                ),
                true,
            );
        }

        let new_entries = cache::list_cache_entries();
        ui.set_cache_entries(cache_entries_to_model(&new_entries));
        update_cache_button(&ui);
        *paths_ref.borrow_mut() = new_entries.iter().map(|e| e.path.clone()).collect::<Vec<_>>();
    });

    let paths_ref = cache_paths;
    let ui_handle = ui.as_weak();
    ui.on_refresh_cache(move || {
        let ui = ui_handle.unwrap();
        let new_entries = cache::list_cache_entries();
        ui.set_cache_entries(cache_entries_to_model(&new_entries));
        update_cache_button(&ui);
        *paths_ref.borrow_mut() = new_entries.iter().map(|e| e.path.clone()).collect::<Vec<_>>();
        set_cache_msg(&ui, "Cache list refreshed.", false);
    });
}
