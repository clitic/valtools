use crate::globals::{
    RIOT_CLIENT_PRIVATE_SETTINGS_PATH, VALTOOLS_ACCOUNTS_DIR, VALTOOLS_CONFIG_PATH,
};
use crate::launcher;
use std::fs;

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

    pub fn save(&self) -> Result<(), String> {
        let path = &*VALTOOLS_CONFIG_PATH;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let contents = self.active.as_deref().unwrap_or("");
        fs::write(path, contents).map_err(|e| format!("Write error: {}", e))
    }
}

pub fn list_accounts() -> Vec<Session> {
    let accounts_dir = &*VALTOOLS_ACCOUNTS_DIR;
    if !accounts_dir.exists() {
        return Vec::new();
    }

    let config = AppConfig::load();
    let active = config.active.as_deref().unwrap_or("");
    let mut accounts = Vec::new();

    if let Ok(entries) = fs::read_dir(accounts_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let label = entry.file_name().to_string_lossy().to_string();
                let backup = entry.path().join("RiotGamesPrivateSettings.yaml");
                if backup.exists() {
                    accounts.push(Session {
                        is_active: label == active,
                        label,
                    });
                }
            }
        }
    }

    accounts.sort_by(|a, b| a.label.cmp(&b.label));

    accounts
}

pub fn prepare_new_login() -> Result<(), String> {
    if launcher::is_riot_client_running() {
        launcher::kill_riot_client_process()?;
    }

    let settings = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;
    if settings.exists() {
        fs::remove_file(settings).map_err(|e| format!("Failed to delete session: {}", e))?;
    }

    let mut config = AppConfig::load();
    config.active = None;
    config.save()?;

    launcher::launch_riot_client()?;
    Ok(())
}

pub fn save_current_session(label: &str) -> Result<(), String> {
    if label.trim().is_empty() {
        return Err("Label cannot be empty".into());
    }

    let source = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;
    if !source.exists() {
        return Err("No session found. Log in via Riot Client first.".into());
    }

    let dest_dir = VALTOOLS_ACCOUNTS_DIR.join(label.trim());
    fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create dir: {}", e))?;

    let dest = dest_dir.join("RiotGamesPrivateSettings.yaml");
    fs::copy(source, &dest).map_err(|e| format!("Failed to copy session: {}", e))?;

    let mut config = AppConfig::load();
    config.active = Some(label.trim().to_string());
    config.save()?;

    Ok(())
}

pub fn switch_to_account(label: &str) -> Result<(), String> {
    let backup = VALTOOLS_ACCOUNTS_DIR
        .join(label)
        .join("RiotGamesPrivateSettings.yaml");
    if !backup.exists() {
        return Err(format!("No backup found for '{}'", label));
    }

    let target = &*RIOT_CLIENT_PRIVATE_SETTINGS_PATH;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::copy(&backup, target).map_err(|e| format!("Failed to restore session: {}", e))?;

    let mut config = AppConfig::load();
    config.active = Some(label.to_string());
    config.save()?;

    Ok(())
}

pub fn delete_account(label: &str) -> Result<(), String> {
    let account_dir = VALTOOLS_ACCOUNTS_DIR.join(label);
    if account_dir.exists() {
        fs::remove_dir_all(&account_dir).map_err(|e| format!("Failed to delete: {}", e))?;
    }

    let mut config = AppConfig::load();
    if config.active.as_deref() == Some(label) {
        config.active = None;
    }
    config.save()?;

    Ok(())
}
