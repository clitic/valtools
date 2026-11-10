use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    scroll::ScrollableElement as _, tag::Tag, ActiveTheme as _, Icon, IconName, Sizable as _,
    TitleBar,
};
use gpui_kit::{
    div, px, rgb, svg, App, Context, IntoElement, ParentElement as _, Render, Styled as _, Window,
};
use std::path::PathBuf;

use crate::{
    cache, launcher, riot,
    session::{self, Account},
    views::{
        account::render_account_row, footer::render_footer, header::render_header,
        shop::ShopDialogView,
    },
};

pub struct AppView {
    accounts: Vec<Account>,
    cache_paths: Vec<PathBuf>,
    cache_size_bytes: u64,
    status_message: String,
    status_is_error: bool,
}

impl AppView {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let accounts = session::list_accounts();
        let cache_entries = cache::list_cache_entries();
        let cache_size_bytes = cache_entries.iter().map(|e| e.size).sum();
        let cache_paths = cache_entries.into_iter().map(|e| e.path).collect();

        Self {
            accounts,
            cache_paths,
            cache_size_bytes,
            status_message: "Ready".into(),
            status_is_error: false,
        }
    }

    pub fn refresh_accounts(&mut self, cx: &mut Context<Self>) {
        self.accounts = session::list_accounts();
        cx.notify();
    }

    pub fn refresh_cache(&mut self, cx: &mut Context<Self>) {
        let cache_entries = cache::list_cache_entries();
        self.cache_size_bytes = cache_entries.iter().map(|e| e.size).sum();
        self.cache_paths = cache_entries.into_iter().map(|e| e.path).collect();
        cx.notify();
    }

    pub fn prepare_new_login(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = session::prepare_new_login() {
            self.status_message = format!("Setup failed: {}", e);
            self.status_is_error = true;
        } else {
            self.status_message =
                "Opening riot client, please log in and save your session.".into();
            self.status_is_error = false;
            self.refresh_accounts(cx);
        }
        cx.notify();
    }

    pub fn save_session(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.status_message = "Saving session".into();
        self.status_is_error = false;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    riot::store::fetch_game_id_from_session().and_then(|game_id| {
                        session::save_current_session(&game_id)?;
                        Ok(game_id)
                    })
                })
                .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(game_id) => {
                        this.status_message = format!("{}'s session saved", game_id);
                        this.status_is_error = false;
                        this.refresh_accounts(cx);
                    }
                    Err(e) => {
                        this.status_message = format!("Save failed: {}", e);
                        this.status_is_error = true;
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn switch_account(&mut self, label: String, cx: &mut Context<Self>) {
        self.status_message = format!("Switching to {}", label);
        self.status_is_error = false;
        cx.notify();

        let label_clone = label.clone();
        cx.spawn(async move |this, cx| {
            let is_running = cx
                .background_executor()
                .spawn(async move { launcher::is_riot_client_running() })
                .await;

            if is_running {
                this.update(cx, |this, cx| {
                    this.status_message = "Closing riot client".into();
                    cx.notify();
                })
                .ok();

                let kill_res = cx
                    .background_executor()
                    .spawn(async move { launcher::kill_riot_client_process() })
                    .await;

                if let Err(e) = kill_res {
                    this.update(cx, |this, cx| {
                        this.status_message = format!("Failed to close Riot Client: {}", e);
                        this.status_is_error = true;
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            }

            this.update(cx, |this, cx| {
                this.status_message = format!("Applying session for {}", label_clone);
                cx.notify();
            })
            .ok();

            let target_label = label_clone.clone();
            let switch_res = cx
                .background_executor()
                .spawn(async move { session::switch_to_account(&target_label) })
                .await;

            if let Err(e) = switch_res {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Switch failed: {}", e);
                    this.status_is_error = true;
                    cx.notify();
                })
                .ok();
                return;
            }

            let launch_res = cx
                .background_executor()
                .spawn(async move { launcher::launch_riot_client() })
                .await;

            if let Err(e) = launch_res {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Launch failed: {}", e);
                    this.status_is_error = true;
                    cx.notify();
                })
                .ok();
            } else {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Switched to {}", label_clone);
                    this.status_is_error = false;
                    this.refresh_accounts(cx);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    pub fn delete_account(&mut self, label: &str, cx: &mut Context<Self>) {
        if let Err(e) = session::delete_account(label) {
            self.status_message = format!("Delete failed: {}", e);
            self.status_is_error = true;
        } else {
            self.status_message = format!("Removed {}", label);
            self.status_is_error = false;
            self.refresh_accounts(cx);
        }
        cx.notify();
    }

    pub fn execute_clean_cache(&mut self, cx: &mut Context<Self>) {
        if self.cache_paths.is_empty() {
            self.status_message = "Cache is already clean".into();
            self.status_is_error = false;
            cx.notify();
            return;
        }

        self.status_message = "Cleaning cache".into();
        self.status_is_error = false;
        cx.notify();

        let paths = self.cache_paths.clone();
        cx.spawn(async move |this, cx| {
            let (deleted, freed, errors) = cx
                .background_executor()
                .spawn(async move { cache::clean_entries(&paths) })
                .await;

            this.update(cx, |this, cx| {
                if errors.is_empty() {
                    this.status_message = format!(
                        "Cleaned {} items ({} freed).",
                        deleted,
                        cache::format_size(freed)
                    );
                    this.status_is_error = false;
                } else {
                    this.status_message = format!(
                        "Cleaned {} items ({} freed). {} failed.",
                        deleted,
                        cache::format_size(freed),
                        errors.len()
                    );
                    this.status_is_error = true;
                }

                this.refresh_cache(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn open_shop_dialog(label: String, window: &mut Window, cx: &mut App) {
        ShopDialogView::open(label, window, cx);
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let entity = cx.entity().clone();
        let cache_size_str = cache::format_size(self.cache_size_bytes);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                TitleBar::new().child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .pl(px(4.0))
                        .child(
                            svg()
                                .path("icons/valorant.svg")
                                .size(px(16.0))
                                .text_color(rgb(0xfa4454)),
                        )
                        .child(
                            div()
                                .font_bold()
                                .text_sm()
                                .text_color(theme.foreground)
                                .child("ValTools"),
                        )
                        .child(
                            Tag::new()
                                .small()
                                .rounded_full()
                                .child(concat!("v", env!("CARGO_PKG_VERSION"))),
                        ),
                ),
            )
            .child(render_header(entity.clone(), theme))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p_4()
                    .gap_2()
                    .overflow_y_scrollbar()
                    .child(if self.accounts.is_empty() {
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .py_16()
                            .gap_2()
                            .child(
                                Icon::new(IconName::CircleUser)
                                    .text_color(theme.muted_foreground).large(),
                            )
                            .child(
                                div()
                                    .font_medium()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("No saved accounts found"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground.opacity(0.8))
                                    .child("Click 'Add Account' to open Riot Client, log in, and save your session."),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .children(self.accounts.iter().enumerate().map(|(idx, acct)| {
                                render_account_row(acct, idx, entity.clone(), theme)
                            }))
                            .into_any_element()
                    }),
            )
            .child(render_footer(
                entity,
                &self.status_message,
                self.status_is_error,
                &cache_size_str,
                theme,
            ))
    }
}
