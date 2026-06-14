use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};

use crate::logger;

const INTERVAL: Duration = Duration::from_secs(10); // TODO: Change to 30 * 60 (30 minutes) for production
const TTL: Duration = Duration::from_secs(60 * 60 * 2); // 2 hours

pub fn debloater_worker() {
    logger::info("Starting debloater worker (30 min interval)");

    let appdata = match std::env::var("LOCALAPPDATA") {
        Ok(v) => v,
        Err(_) => return,
    };

    let folders: Vec<PathBuf> = vec![
        // Known locations for temporary files and caches that can be safely cleaned up
        PathBuf::from(&appdata).join("Temp"),
        PathBuf::from(&appdata).join("Microsoft\\Windows\\INetCache"),
        PathBuf::from(&appdata).join("Microsoft\\Windows\\WebCache"),
        PathBuf::from(&appdata).join("Microsoft\\Windows\\Explorer"),
    ];

    let file_deletion_errors: HashSet<PathBuf> = HashSet::new();

    thread::spawn(move || {
        loop {
            run_cleanup(&folders, file_deletion_errors.clone());
            thread::sleep(INTERVAL);
        }
    });
}

fn run_cleanup(folders: &[PathBuf], mut file_deletion_errors: HashSet<PathBuf>) {
    let now = SystemTime::now();

    for folder in folders {
        let Ok(entries) = fs::read_dir(folder) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            let Ok(meta) = entry.metadata() else {
                continue;
            };

            let Ok(modified) = meta.modified() else {
                continue;
            };

            if now.duration_since(modified).unwrap_or_default() > TTL {
                // Skip known files that have previously failed to delete
                if file_deletion_errors.contains(&path) {
                    continue;
                }

                // Handle file deletion errors gracefully
                if path.is_file() {
                    if let Err(e) = fs::remove_file(&path) {
                        logger::error(&format!("Failed to delete file {}: {}", path.display(), e));
                        file_deletion_errors.insert(path);
                    }
                } else if path.is_dir() {
                    if let Err(e) = fs::remove_dir_all(&path) {
                        logger::error(&format!(
                            "Failed to delete directory {}: {}",
                            path.display(),
                            e
                        ));
                        file_deletion_errors.insert(path);
                    }
                }
            }
        }
    }
}
