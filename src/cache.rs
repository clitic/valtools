use std::{fs, path::Path, path::PathBuf};

pub struct CacheEntry {
    pub path: PathBuf,
    pub size: u64,
}

pub fn list_cache_entries() -> Vec<CacheEntry> {
    let dir = &*crate::globals::VALORANT_LOCAL_DIR;
    let mut entries = Vec::new();

    if !dir.exists() {
        return entries;
    }

    // Scan top-level and Saved subdirectory
    for scan_dir in [dir.to_path_buf(), dir.join("Saved")] {
        if !scan_dir.exists() {
            continue;
        }
        if let Ok(dir_entries) = fs::read_dir(&scan_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();

                // Skip the "Saved" dir itself since we scan inside it
                if path == dir.join("Saved") {
                    continue;
                }

                let size = if path.is_file() {
                    path.metadata().map(|m| m.len()).unwrap_or(0)
                } else if path.is_dir() {
                    dir_size(&path)
                } else {
                    continue;
                };

                entries.push(CacheEntry { path, size });
            }
        }
    }

    entries.sort_by(|a, b| b.size.cmp(&a.size));
    entries
}

pub fn clean_entries(paths: &[PathBuf]) -> (usize, u64, Vec<String>) {
    let mut deleted = 0usize;
    let mut freed = 0u64;
    let mut errors = Vec::new();

    for path in paths {
        if !path.exists() {
            continue;
        }

        if path.is_file() {
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            if let Err(e) = fs::remove_file(path) {
                errors.push(format!("{}: {}", path.display(), e));
            } else {
                deleted += 1;
                freed += size;
            }
        } else if path.is_dir() {
            let size = dir_size(path);
            if let Err(e) = fs::remove_dir_all(path) {
                errors.push(format!("{}: {}", path.display(), e));
            } else {
                deleted += 1;
                freed += size;
            }
        }
    }

    (deleted, freed, errors)
}

pub fn dir_size(path: &Path) -> u64 {
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
