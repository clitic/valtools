use crate::{anyhow, bail, error::Result};
use std::{os::windows::process::CommandExt, path::Path, process::Command};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn launch_riot_client() -> Result<()> {
    let client = Path::new("C:\\Riot Games\\Riot Client\\RiotClientServices.exe");

    if !client.exists() {
        bail!("Could not find RiotClientServices.exe");
    }

    Command::new(client)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|_| anyhow!("Failed to launch Riot Client"))?;
    Ok(())
}

pub fn is_riot_client_running() -> bool {
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq RiotClientServices.exe", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("RiotClientServices.exe"))
        .unwrap_or(false)
}

pub fn kill_riot_client_process() -> Result<()> {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "RiotClientServices.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    std::thread::sleep(std::time::Duration::from_secs(2));

    if is_riot_client_running() {
        bail!("Riot Client is still running, try again");
    }

    Ok(())
}
