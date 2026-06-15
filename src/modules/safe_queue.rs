use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;
use std::io::{ self, Write };
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::sync::atomic::{ AtomicBool, Ordering };

use crate::logger;

use windows_sys::Win32::UI::WindowsAndMessaging::{ MessageBoxW, MB_ICONINFORMATION, MB_OK };

static SCAN_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct QueueItem {
    pub path: String,
    pub size_bytes: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct StagedQueue {
    pub items: Vec<QueueItem>,
}

fn get_queue_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\".to_string());
    let mut path = PathBuf::from(appdata).join("WinSentry");
    let _ = fs::create_dir_all(&path);
    path.push("queue.json");
    path
}

pub fn load_queue() -> StagedQueue {
    let path = get_queue_path();
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(queue) = serde_json::from_str(&data) {
            return queue;
        }
    }
    StagedQueue::default()
}

pub fn save_queue(queue: &StagedQueue) {
    let path = get_queue_path();
    if let Ok(json) = serde_json::to_string_pretty(queue) {
        let _ = fs::write(path, json);
    }
}

fn format_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let bytes_f = bytes as f64;
    if bytes_f >= GIB {
        format!("{:.2} GB", bytes_f / GIB)
    } else if bytes_f >= MIB {
        format!("{:.2} MB", bytes_f / MIB)
    } else if bytes_f >= KIB {
        format!("{:.2} KB", bytes_f / KIB)
    } else {
        format!("{} B", bytes)
    }
}

// -----------------------------------------------------------------
// DISCOVERY PHASE
// -----------------------------------------------------------------
pub fn run_discovery_scan(
    target_roots: Vec<String>,
    targets_to_match: HashSet<String>,
    is_background: bool
) {
    if is_background {
        if
            SCAN_IN_PROGRESS.compare_exchange(
                false,
                true,
                Ordering::SeqCst,
                Ordering::SeqCst
            ).is_err()
        {
            logger::warn("Scan already in progress. New scan request ignored.");
            return;
        }

        std::thread::spawn(|| {
            unsafe {
                let title: Vec<u16> = "WinSentry\0".encode_utf16().collect();
                let message: Vec<u16> =
                    "Scan started in background. You will be notified when it completes."
                        .encode_utf16()
                        .chain(std::iter::once(0))
                        .collect();
                MessageBoxW(0, message.as_ptr(), title.as_ptr(), MB_ICONINFORMATION | MB_OK);
            }
        });
    }

    logger::info("Scan started in background. You will be notified when it completes.");
    let mut queue = StagedQueue::default();
    let mut total_bloat_bytes: u64 = 0;

    for root in target_roots {
        let mut clean_root = root.replace("\\\\", "\\").replace("/", "\\").trim().to_string();

        while clean_root.ends_with('\\') {
            clean_root.pop();
        }

        let root_path = PathBuf::from(&clean_root);

        if root_path.exists() && root_path.is_dir() {
            scan_directory_recursive(
                &root_path,
                &targets_to_match,
                &mut queue,
                &mut total_bloat_bytes
            );
        }
    }

    save_queue(&queue);

    let size_string = format_size(total_bloat_bytes);

    if is_background {
        SCAN_IN_PROGRESS.store(false, Ordering::SeqCst);

        if queue.items.is_empty() {
            std::thread::spawn(|| {
                unsafe {
                    let title: Vec<u16> = "WinSentry\0".encode_utf16().collect();
                    let message: Vec<u16> =
                        "Scan completed: no bloat items found in the specified directories."
                            .encode_utf16()
                            .chain(std::iter::once(0))
                            .collect();
                    MessageBoxW(0, message.as_ptr(), title.as_ptr(), MB_ICONINFORMATION | MB_OK);
                }
            });
            return;
        }

        // LANCIO DIRETTO BULLETPROOF: Riapre WinSentry con --purge in una nuova console visibile
        std::thread::spawn(move || {
            if let Ok(current_exe) = std::env::current_exe() {
                const CREATE_NEW_CONSOLE: u32 = 0x00000010;

                let mut terminal_window = Command::new(current_exe);
                terminal_window.arg("--purge").creation_flags(CREATE_NEW_CONSOLE);

                if let Err(e) = terminal_window.spawn() {
                    logger::error(&format!("Failed to auto-open purge terminal: {}", e));
                }
            }
        });
    } else {
        println!(
            "\n[WinSentry Bloat Alert]: Found {} of bloat in {} projects.",
            size_string,
            queue.items.len()
        );
        println!("Run `winsentry --purge` to review and remove bloat.");
    }
}

fn scan_directory_recursive(
    dir: &PathBuf,
    targets: &HashSet<String>,
    queue: &mut StagedQueue,
    total_bytes: &mut u64
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() || (file_type.is_symlink() && path.is_dir()) {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if targets.contains(&dir_name.to_lowercase()) {
                    let size = get_dir_size(&path);
                    *total_bytes += size;
                    queue.items.push(QueueItem {
                        path: path.to_string_lossy().into_owned(),
                        size_bytes: size,
                    });

                    logger::info(&format!("[✓ FOUND] {} ({})", path.display(), format_size(size)));
                    continue;
                }
            }
            scan_directory_recursive(&path, targets, queue, total_bytes);
        }
    }
}

fn get_dir_size(path: &PathBuf) -> u64 {
    let mut total = 0;
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            total += get_dir_size(&entry.path());
        } else if let Ok(meta) = entry.metadata() {
            total += meta.len();
        }
    }
    total
}

// -----------------------------------------------------------------
// INTERACTIVE PURGE PHASE
// -----------------------------------------------------------------
pub fn interactive_purge() {
    let mut queue = load_queue();
    if queue.items.is_empty() {
        println!(
            "The safe queue is currently empty. No bloat detected or all items have been removed."
        );
        return;
    }

    let total_queue_bytes: u64 = queue.items
        .iter()
        .map(|item| item.size_bytes)
        .sum();

    println!("\n=================== WinSentry Safe Queue Breakdown ===================");
    let max_path_len = queue.items
        .iter()
        .map(|item| item.path.len())
        .max()
        .unwrap_or(30);

    for (index, item) in queue.items.iter().enumerate() {
        println!(
            "[ ] {:2}. {:<width$} ({})",
            index + 1,
            item.path,
            format_size(item.size_bytes),
            width = max_path_len + 2
        );
    }

    println!("----------------------------------------------------------------------");
    println!("{:<width$} Total: {}", "", format_size(total_queue_bytes), width = max_path_len + 6);
    println!("======================================================================");

    print!("\nSelect folders to purge (e.g., 1,3), 'all' for all, or 'q' to quit: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let input = input.trim().to_lowercase();

    if input == "q" || input.is_empty() {
        println!("Operation cancelled.");
        return;
    }

    let mut indices_to_delete = Vec::new();

    if input == "all" {
        indices_to_delete = (0..queue.items.len()).collect();
    } else {
        for part in input.split(',') {
            if let Ok(num) = part.trim().parse::<usize>() {
                if num > 0 && num <= queue.items.len() {
                    indices_to_delete.push(num - 1);
                }
            }
        }
    }

    if indices_to_delete.is_empty() {
        println!("Invalid selection.");
        return;
    }

    indices_to_delete.sort_by(|a, b| b.cmp(a));

    println!("\nStarting safe removal of selected folders...");
    let mut remaining_items = queue.items.clone();
    let mut success_count = 0;
    let mut fail_count = 0;

    for idx in indices_to_delete {
        let item = &queue.items[idx];
        let path = PathBuf::from(&item.path);

        match fs::remove_dir_all(&path) {
            Ok(_) => {
                println!("[✓ REMOVED] {}", item.path);
                remaining_items.remove(idx);
                success_count += 1;
            }
            Err(e) => {
                println!("[X FAILED]  {} -> Error: {}", item.path, e);
                fail_count += 1;
            }
        }
    }

    queue.items = remaining_items;
    save_queue(&queue);

    println!("\n=== Operation Summary ===");
    println!("Successfully purged: {} directories", success_count);
    if fail_count > 0 {
        println!("Failed to remove: {} folders", fail_count);
    }
}
