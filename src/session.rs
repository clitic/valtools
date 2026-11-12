use crate::error::{anyhow, bail, Result};
use crate::globals::{RIOT_CLIENT_PRIVATE_SETTINGS_PATH, VALTOOLS_CONFIG_PATH, VALTOOLS_DIR};
use crate::launcher;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Session {
    pub label: String,
    pub is_active: bool,
}

pub type Account = Session;

#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub active: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        let path = &*VALTOOLS_CONFIG_PATH;
        if let Ok(contents) = fs::read_to_string(path) {
            let trimmed = contents.trim();
            if !trimmed.is_empty() {
                return AppConfig {
                    active: Some(trimmed.to_string()),
                };
            }
        }
        AppConfig::default()
    }

    pub fn save(&self) -> Result<()> {
        let path = &*VALTOOLS_CONFIG_PATH;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let contents = self.active.as_deref().unwrap_or("");
        fs::write(path, contents).map_err(|_| anyhow!("Failed to save active account"))
    }
}

pub fn account_path(label: &str) -> PathBuf {
    VALTOOLS_DIR.join(format!("{}.yaml", label.trim()))
}

pub fn list_accounts() -> Vec<Session> {
    let accounts_dir = &*VALTOOLS_DIR;
    if !accounts_dir.exists() {
        return Vec::new();
    }

    let config = AppConfig::load();
    let active = config.active.as_deref().unwrap_or("");
    let mut accounts = Vec::new();

    if let Ok(entries) = fs::read_dir(accounts_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                    if let Some(label) = path.file_stem().and_then(|s| s.to_str()) {
                        let label = label.to_string();
                        accounts.push(Session {
                            is_active: label == active,
                            label,
                        });
                    }
                }
            }
        }
    }

    accounts.sort_by(|a, b| a.label.cmp(&b.label));

    accounts
}

pub fn prepare_new_login() -> Result<()> {
    if launcher::is_riot_client_running() {
        launcher::kill_riot_client_process()?;
    }

    let settings = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;
    if settings.exists() {
        fs::remove_file(settings).map_err(|_| anyhow!("Failed to clear current Riot session"))?;
    }

    let mut config = AppConfig::load();
    config.active = None;
    config.save()?;

    launcher::launch_riot_client()?;
    Ok(())
}

pub fn save_current_session(label: &str) -> Result<()> {
    let label = label.trim();
    if label.is_empty() {
        bail!("Account name cannot be empty");
    }

    let source = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;
    if !source.exists() {
        bail!("No active Riot session found");
    }

    fs::create_dir_all(&*VALTOOLS_DIR)
        .map_err(|_| anyhow!("Failed to create ValTools directory"))?;

    let dest = account_path(label);
    fs::copy(source, &dest).map_err(|_| anyhow!("Failed to save session for {}", label))?;

    let mut config = AppConfig::load();
    config.active = Some(label.to_string());
    config.save()?;

    Ok(())
}

pub fn switch_to_account(label: &str) -> Result<()> {
    let backup = account_path(label);
    if !backup.exists() {
        bail!("No saved session found for {}", label);
    }

    let target = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::copy(&backup, target).map_err(|_| anyhow!("Failed to apply session for {}", label))?;

    let mut config = AppConfig::load();
    config.active = Some(label.to_string());
    config.save()?;

    Ok(())
}

pub fn delete_account(label: &str) -> Result<()> {
    let path = account_path(label);
    if path.exists() {
        fs::remove_file(&path).map_err(|_| anyhow!("Failed to delete account {}", label))?;
    }

    let mut config = AppConfig::load();
    if config.active.as_deref() == Some(label) {
        config.active = None;
    }
    config.save()?;

    Ok(())
}
