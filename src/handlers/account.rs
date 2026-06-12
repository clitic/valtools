use slint::{ComponentHandle, Timer};

use crate::{accounts, client, riot, AppWindow};
use super::{set_status, refresh_accounts_model};

pub(crate) fn setup(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_prepare_new_login(move || {
        let ui = ui_handle.unwrap();
        if let Err(e) = accounts::prepare_new_login() {
            set_status(&ui, &format!("{}", e), true);
        } else {
            set_status(
                &ui,
                "Riot Client opened - log in and click Save Session.",
                false,
            );
            refresh_accounts_model(&ui);
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_save_session(move || {
        let ui = ui_handle.unwrap();
        set_status(&ui, "Saving session...", false);

        let ui_weak = ui.as_weak();
        std::thread::spawn(move || {
            let result = riot::shop::fetch_game_id_from_session().and_then(|game_id| {
                accounts::save_current_session(&game_id)?;
                Ok(game_id)
            });

            let _ = ui_weak.upgrade_in_event_loop(move |ui| match result {
                Ok(game_id) => {
                    set_status(&ui, &format!("Session saved as '{}'.", game_id), false);
                    refresh_accounts_model(&ui);
                }
                Err(e) => set_status(&ui, &e.to_string(), true),
            });
        });
    });

    let ui_handle = ui.as_weak();
    ui.on_switch_account(move |label| {
        let ui = ui_handle.unwrap();
        let label_str = label.to_string();

        let needs_kill = client::is_riot_client_running();

        if needs_kill {
            set_status(&ui, "Closing Riot Client...", false);
        } else {
            set_status(&ui, &format!("Switching to '{}'...", label_str), false);
        }

        let ui_weak = ui.as_weak();
        let label_owned = label_str.clone();
        Timer::single_shot(std::time::Duration::from_millis(100), move || {
            let ui = ui_weak.unwrap();

            if needs_kill {
                if let Err(e) = client::kill_riot_client_process() {
                    set_status(&ui, &format!("{}", e), true);
                    return;
                }
                set_status(&ui, &format!("Applying session for '{}'...", label_owned), false);
            }

            let ui_weak = ui.as_weak();
            let label_owned2 = label_owned.clone();
            Timer::single_shot(std::time::Duration::from_millis(100), move || {
                let ui = ui_weak.unwrap();

                if let Err(e) = accounts::switch_to_account(&label_owned2) {
                    set_status(&ui, &format!("{}", e), true);
                    return;
                }

                refresh_accounts_model(&ui);
                set_status(
                    &ui,
                    &format!("Launching Riot Client for '{}'...", label_owned2),
                    false,
                );

                let ui_weak = ui.as_weak();
                let label_owned3 = label_owned2.clone();
                Timer::single_shot(std::time::Duration::from_millis(100), move || {
                    let ui = ui_weak.unwrap();

                    if let Err(e) = client::launch_riot_client() {
                        set_status(
                            &ui,
                            &format!("Failed to launch Riot Client: {}", e),
                            true,
                        );
                    } else {
                        set_status(
                            &ui,
                            &format!("Riot Client launched as '{}'.", label_owned3),
                            false,
                        );
                    }
                });
            });
        });
    });

    let ui_handle = ui.as_weak();
    ui.on_delete_account(move |label| {
        let ui = ui_handle.unwrap();
        let label_str = label.to_string();
        if let Err(e) = accounts::delete_account(&label_str) {
            set_status(&ui, &format!("{}", e), true);
        } else {
            set_status(&ui, &format!("Account '{}' removed.", label_str), false);
            refresh_accounts_model(&ui);
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_launch_valorant(move |label| {
        let ui = ui_handle.unwrap();
        let label_str = label.to_string();

        let needs_kill = client::is_riot_client_running();

        if needs_kill {
            set_status(&ui, "Closing Riot Client...", false);
        } else {
            set_status(&ui, &format!("Switching to '{}'...", label_str), false);
        }

        let ui_weak = ui.as_weak();
        let label_owned = label_str.clone();
        Timer::single_shot(std::time::Duration::from_millis(100), move || {
            let ui = ui_weak.unwrap();

            if needs_kill {
                if let Err(e) = client::kill_riot_client_process() {
                    set_status(&ui, &format!("{}", e), true);
                    return;
                }
                set_status(&ui, &format!("Applying session for '{}'...", label_owned), false);
            }

            let ui_weak = ui.as_weak();
            let label_owned2 = label_owned.clone();
            Timer::single_shot(std::time::Duration::from_millis(100), move || {
                let ui = ui_weak.unwrap();

                if let Err(e) = accounts::switch_to_account(&label_owned2) {
                    set_status(&ui, &format!("{}", e), true);
                    return;
                }

                refresh_accounts_model(&ui);
                set_status(
                    &ui,
                    &format!("Launching Valorant for '{}'...", label_owned2),
                    false,
                );

                let ui_weak = ui.as_weak();
                let label_owned3 = label_owned2.clone();
                Timer::single_shot(std::time::Duration::from_millis(100), move || {
                    let ui = ui_weak.unwrap();

                    if let Err(e) = client::launch_valorant() {
                        set_status(
                            &ui,
                            &format!("Failed to launch Valorant: {}", e),
                            true,
                        );
                    } else {
                        set_status(
                            &ui,
                            &format!("Valorant launched as '{}'.", label_owned3),
                            false,
                        );
                    }
                });
            });
        });
    });
}
