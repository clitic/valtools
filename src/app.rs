use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    scroll::ScrollableElement as _,
    ActiveTheme as _, Icon, IconName, Sizable as _, TitleBar,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, rgb, svg, App, Context, IntoElement,
    ParentElement as _, Render, Styled as _, Window,
};
use std::{path::PathBuf, process::Command};

use crate::{
    accounts::{self, Account},
    cache, client, riot,
    views::{account_row::render_account_row, cache_dialog::open_cache_dialog, shop_dialog::ShopDialogView},
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
        let accounts = accounts::list_accounts();
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
        self.accounts = accounts::list_accounts();
        cx.notify();
    }

    pub fn refresh_cache(&mut self, cx: &mut Context<Self>) {
        let cache_entries = cache::list_cache_entries();
        self.cache_size_bytes = cache_entries.iter().map(|e| e.size).sum();
        self.cache_paths = cache_entries.into_iter().map(|e| e.path).collect();
        cx.notify();
    }

    pub fn prepare_new_login(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = accounts::prepare_new_login() {
            self.status_message = e.to_string();
            self.status_is_error = true;
        } else {
            self.status_message = "Riot Client opened - log in and click Save Session.".into();
            self.status_is_error = false;
            self.refresh_accounts(cx);
        }
        cx.notify();
    }

    pub fn save_session(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.status_message = "Saving session...".into();
        self.status_is_error = false;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    riot::shop::fetch_game_id_from_session().and_then(|game_id| {
                        accounts::save_current_session(&game_id)?;
                        Ok(game_id)
                    })
                })
                .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(game_id) => {
                        this.status_message = format!("Session saved as '{}'.", game_id);
                        this.status_is_error = false;
                        this.refresh_accounts(cx);
                    }
                    Err(e) => {
                        this.status_message = e.to_string();
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
        self.status_message = format!("Switching to '{}'...", label);
        self.status_is_error = false;
        cx.notify();

        let label_clone = label.clone();
        cx.spawn(async move |this, cx| {
            let is_running = cx
                .background_executor()
                .spawn(async move { client::is_riot_client_running() })
                .await;

            if is_running {
                this.update(cx, |this, cx| {
                    this.status_message = "Closing running Riot Client...".into();
                    cx.notify();
                })
                .ok();

                let kill_res = cx
                    .background_executor()
                    .spawn(async move { client::kill_riot_client_process() })
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
                this.status_message = format!("Applying session for '{}'...", label_clone);
                cx.notify();
            })
            .ok();

            let target_label = label_clone.clone();
            let switch_res = cx
                .background_executor()
                .spawn(async move { accounts::switch_to_account(&target_label) })
                .await;

            if let Err(e) = switch_res {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Failed to switch session: {}", e);
                    this.status_is_error = true;
                    cx.notify();
                })
                .ok();
                return;
            }

            let launch_res = cx
                .background_executor()
                .spawn(async move { client::launch_riot_client() })
                .await;

            if let Err(e) = launch_res {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Failed to launch Riot Client: {}", e);
                    this.status_is_error = true;
                    cx.notify();
                })
                .ok();
            } else {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Riot Client launched as '{}'.", label_clone);
                    this.status_is_error = false;
                    this.refresh_accounts(cx);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    pub fn launch_valorant(&mut self, label: String, cx: &mut Context<Self>) {
        self.status_message = format!("Switching to '{}'...", label);
        self.status_is_error = false;
        cx.notify();

        let label_clone = label.clone();
        cx.spawn(async move |this, cx| {
            let is_running = cx
                .background_executor()
                .spawn(async move { client::is_riot_client_running() })
                .await;

            if is_running {
                this.update(cx, |this, cx| {
                    this.status_message = "Closing running Riot Client...".into();
                    cx.notify();
                })
                .ok();

                let kill_res = cx
                    .background_executor()
                    .spawn(async move { client::kill_riot_client_process() })
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
                this.status_message = format!("Applying session for '{}'...", label_clone);
                cx.notify();
            })
            .ok();

            let target_label = label_clone.clone();
            let switch_res = cx
                .background_executor()
                .spawn(async move { accounts::switch_to_account(&target_label) })
                .await;

            if let Err(e) = switch_res {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Failed to switch session: {}", e);
                    this.status_is_error = true;
                    cx.notify();
                })
                .ok();
                return;
            }

            let launch_res = cx
                .background_executor()
                .spawn(async move { client::launch_valorant() })
                .await;

            if let Err(e) = launch_res {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Failed to launch Valorant: {}", e);
                    this.status_is_error = true;
                    cx.notify();
                })
                .ok();
            } else {
                this.update(cx, |this, cx| {
                    this.status_message = format!("Valorant launched as '{}'.", label_clone);
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
        if let Err(e) = accounts::delete_account(label) {
            self.status_message = format!("Failed to delete: {}", e);
            self.status_is_error = true;
        } else {
            self.status_message = format!("Account '{}' removed.", label);
            self.status_is_error = false;
            self.refresh_accounts(cx);
        }
        cx.notify();
    }

    pub fn execute_clean_cache(&mut self, cx: &mut Context<Self>) {
        if self.cache_paths.is_empty() {
            self.status_message = "No cache items found to clean.".into();
            self.status_is_error = false;
            cx.notify();
            return;
        }

        self.status_message = "Cleaning cache...".into();
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
                        "Cleaned {} items, {} freed.",
                        deleted,
                        cache::format_size(freed)
                    );
                    this.status_is_error = false;
                } else {
                    this.status_message = format!(
                        "Cleaned {} items ({} freed). {} item(s) could not be removed.",
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
            // 1. Custom Light TitleBar
            .child(
                TitleBar::new().child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
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
                            Badge::new()
                                .child("v0.1.1"),
                        ),
                ),
            )
            // 2. Action Header Toolbar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .bg(theme.secondary.opacity(0.3))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .font_bold()
                                    .text_base()
                                    .text_color(theme.foreground)
                                    .child("Account Switcher"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child("Switch Riot accounts without logging out"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            // Add New Account button
                            .child(
                                Button::new("btn_add_account")
                                    .primary()
                                    .small()
                                    .icon(IconName::Plus)
                                    .label("Add Account")
                                    .tooltip("Log in with a new account via Riot Client")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.prepare_new_login(window, cx);
                                    })),
                            )
                            // Save Session button
                            .child(
                                Button::new("btn_save_session")
                                    .outline()
                                    .small()
                                    .icon(Icon::new(gpui_kit::assets::IconName::Save))
                                    .label("Save Session")
                                    .tooltip("Save the current logged-in Riot session")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_session(window, cx);
                                    })),
                            )
                            // Daily Shop button (for active or first account)
                            .when_some(
                                self.accounts.iter().find(|a| a.is_active).or_else(|| self.accounts.first()),
                                |this, active_acct| {
                                    let active_label = active_acct.label.clone();
                                    this.child(
                                        Button::new("btn_toolbar_shop")
                                            .outline()
                                            .small()
                                            .icon(Icon::new(gpui_kit::assets::IconName::Store))
                                            .label("Daily Shop")
                                            .tooltip(format!("View Daily Shop for '{}'", active_label))
                                            .on_click(move |_, window, cx| {
                                                AppView::open_shop_dialog(active_label.clone(), window, cx);
                                            }),
                                    )
                                },
                            )
                            // Clean Cache Button (showing size in MB)
                            .child(
                                Button::new("btn_clean_cache")
                                    .outline()
                                    .small()
                                    .icon(Icon::new(gpui_kit::assets::IconName::Trash))
                                    .label(format!("Clean Cache ({})", cache_size_str))
                                    .tooltip("Clean Valorant temporary cache files")
                                    .on_click({
                                        let entity = entity.clone();
                                        let size_str = cache_size_str.clone();
                                        move |_, window, cx| {
                                            open_cache_dialog(size_str.clone(), entity.clone(), window, cx);
                                        }
                                    }),
                            ),
                    ),
            )
            // 3. Central Accounts List
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
                                    .text_color(theme.muted_foreground),
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
            // 4. Status Bar & External Links Footer
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(36.0))
                    .px_4()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.secondary.opacity(0.3))
                    // Status feedback
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Icon::new(if self.status_is_error {
                                    IconName::TriangleAlert
                                } else {
                                    IconName::Check
                                })
                                .xsmall()
                                .text_color(if self.status_is_error {
                                    theme.danger
                                } else {
                                    theme.primary
                                }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if self.status_is_error {
                                        theme.danger
                                    } else {
                                        theme.muted_foreground
                                    })
                                    .child(self.status_message.clone()),
                            ),
                    )
                    // External Links
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            // GitHub link
                            .child(
                                Button::new("btn_github")
                                    .ghost()
                                    .xsmall()
                                    .label("GitHub")
                                    .tooltip("Open GitHub source repository")
                                    .child(
                                        svg()
                                            .path("icons/github.svg")
                                            .size(px(14.0))
                                            .text_color(theme.foreground),
                                    )
                                    .on_click(|_, _, cx| {
                                        cx.open_url("https://github.com/clitic/valtools");
                                        let _ = Command::new("cmd")
                                            .args(["/C", "start", "", "https://github.com/clitic/valtools"])
                                            .spawn();
                                    }),
                            )
                            // Report Issues link
                            .child(
                                Button::new("btn_issues")
                                    .ghost()
                                    .xsmall()
                                    .icon(IconName::TriangleAlert)
                                    .label("Report Issue")
                                    .tooltip("Report an issue on GitHub")
                                    .on_click(|_, _, cx| {
                                        cx.open_url("https://github.com/clitic/valtools/issues");
                                        let _ = Command::new("cmd")
                                            .args(["/C", "start", "", "https://github.com/clitic/valtools/issues"])
                                            .spawn();
                                    }),
                            ),
                    ),
            )
    }
}
