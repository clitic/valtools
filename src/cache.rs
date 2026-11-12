use crate::error::{anyhow, Result};
use std::{fs, path::Path};

pub fn total_size() -> u64 {
    dir_size(&crate::globals::VALORANT_LOCAL_DIR)
}

pub fn clean() -> Result<u64> {
    let dir = &*crate::globals::VALORANT_LOCAL_DIR;
    if !dir.exists() {
        return Ok(0);
    }

    let size = dir_size(dir);
    fs::remove_dir_all(dir).map_err(|_| anyhow!("Failed to clean cache"))?;
    Ok(size)
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    total += entry.metadata().map(|m| m.len()).unwrap_or(0);
                } else if ft.is_dir() {
                    total += dir_size(&entry.path());
                }
            }
        }
    }
    total
}

pub fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}
