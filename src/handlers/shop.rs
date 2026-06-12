use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{accounts, riot, AppWindow, ShopResultData};

fn push_shop_result(
    ui: &AppWindow,
    label: &str,
    game_id: &str,
    vp: i32,
    reset_time: &str,
    skins: &[(String, i32, String)],
    has_error: bool,
    error_msg: &str,
) {
    let skin = |idx: usize| -> (SharedString, i32, slint::Image) {
        skins
            .get(idx)
            .map(|(n, c, p)| {
                let img = if p.is_empty() {
                    slint::Image::default()
                } else {
                    slint::Image::load_from_path(std::path::Path::new(p)).unwrap_or_default()
                };
                (SharedString::from(n.as_str()), *c, img)
            })
            .unwrap_or_default()
    };
    let (s1n, s1c, s1i) = skin(0);
    let (s2n, s2c, s2i) = skin(1);
    let (s3n, s3c, s3i) = skin(2);
    let (s4n, s4c, s4i) = skin(3);
    let model = VecModel::default();
    model.push(ShopResultData {
        label: SharedString::from(label),
        game_id: SharedString::from(game_id),
        vp,
        reset_time: SharedString::from(reset_time),
        skin1_name: s1n,
        skin1_cost: s1c,
        skin1_icon: s1i,
        skin2_name: s2n,
        skin2_cost: s2c,
        skin2_icon: s2i,
        skin3_name: s3n,
        skin3_cost: s3c,
        skin3_icon: s3i,
        skin4_name: s4n,
        skin4_cost: s4c,
        skin4_icon: s4i,
        has_error,
        error_msg: SharedString::from(error_msg),
    });
    ui.set_shop_results(ModelRc::new(model));
}

pub(crate) fn setup(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_refresh_shop_accounts(move || {
        let ui = ui_handle.unwrap();
        let accts = accounts::list_accounts();
        let labels = VecModel::default();
        let mut active_idx: i32 = 0;
        for (i, a) in accts.iter().enumerate() {
            labels.push(SharedString::from(&a.label));
            if a.is_active {
                active_idx = i as i32;
            }
        }
        ui.set_shop_account_labels(ModelRc::new(labels));
        ui.set_shop_selected_index(active_idx);
    });

    let ui_handle = ui.as_weak();
    ui.on_fetch_shop(move || {
        let ui = ui_handle.unwrap();
        let labels = ui.get_shop_account_labels();
        let idx = ui.get_shop_selected_index() as usize;
        let download_images = ui.get_shop_download_images();

        let label = match labels.row_data(idx) {
            Some(l) => l.to_string(),
            None => {
                ui.set_shop_status(SharedString::from("No account selected."));
                ui.set_shop_is_error(true);
                return;
            }
        };

        ui.set_shop_loading(true);
        ui.set_shop_results(ModelRc::new(VecModel::default()));
        ui.set_shop_status(SharedString::from("Preparing..."));
        ui.set_shop_is_error(false);

        let ui_weak = ui.as_weak();
        std::thread::spawn(move || {
            let ui_weak_progress = ui_weak.clone();
            let set_status = move |msg: &str| {
                let msg = SharedString::from(msg);
                let _ = ui_weak_progress.upgrade_in_event_loop(move |ui| {
                    ui.set_shop_status(msg);
                });
            };

            match riot::shop::fetch_account_store(&label, "ap", &set_status) {
                Ok(mut st) => {
                    // Download images if the switch is on
                    if download_images {
                        let total = st
                            .skins
                            .iter()
                            .filter(|sk| sk.icon_path.is_none() && !sk.icon_url.is_empty())
                            .count();
                        for (done, i) in (0..st.skins.len())
                            .filter(|i| {
                                st.skins[*i].icon_path.is_none()
                                    && !st.skins[*i].icon_url.is_empty()
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                            .enumerate()
                        {
                            set_status(&format!(
                                "Downloading image {}/{}...",
                                done + 1,
                                total
                            ));
                            st.skins[i].icon_path =
                                riot::shop::download_single_image(&st.skins[i].icon_url);
                        }
                    }

                    // Build final skin data
                    let skins = st
                        .skins
                        .iter()
                        .map(|sk| {
                            let path = sk
                                .icon_path
                                .as_ref()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default();
                            (sk.name.clone(), sk.cost as i32, path)
                        })
                        .collect::<Vec<_>>();

                    let game_id = st.game_id.clone();
                    let vp = st.vp as i32;
                    let reset_time = riot::shop::format_reset_time(st.reset_secs);

                    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                        push_shop_result(
                            &ui, &label, &game_id, vp, &reset_time, &skins, false, "",
                        );
                        ui.set_shop_status(SharedString::from("Shop loaded."));
                        ui.set_shop_loading(false);
                        ui.set_shop_is_error(false);
                    });
                }
                Err(e) => {
                    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                        push_shop_result(&ui, &label, "", 0, "", &[], true, &e.to_string());
                        ui.set_shop_status(SharedString::from("Failed to load shop."));
                        ui.set_shop_loading(false);
                        ui.set_shop_is_error(true);
                    });
                }
            }
        });
    });
}
