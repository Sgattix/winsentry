use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::{configuration::get_win_exp_reboot_hotkey, logger};
use win_hotkeys::{HotkeyManager, VKey};

static RUNNING: AtomicBool = AtomicBool::new(false);

pub fn taskbar_refresh(config_file_path: String) {
    std::thread::spawn(move || {
        let mut hotkey_manager = HotkeyManager::new();

        let key_str = get_win_exp_reboot_hotkey(&config_file_path).unwrap_or("W".to_string());

        let key = VKey::from_str(&key_str).unwrap_or(VKey::W);

        hotkey_manager
            .register_hotkey(key, &[VKey::Control, VKey::Menu], || {
                if RUNNING.swap(true, Ordering::SeqCst) {
                    logger::info("Explorer restart already running");
                    return;
                }

                logger::info("Rebooting Windows Explorer...");

                // 1. Kill Explorer
                let _ = std::process::Command::new("cmd")
                    .args(&["/C", "taskkill /F /IM explorer.exe"])
                    .status();

                std::thread::sleep(Duration::from_millis(700));

                // 2. Optional icon cache cleanup
                let _ = std::process::Command::new("cmd")
                    .args(&["/C", "ie4uinit.exe -ClearIconCache"])
                    .status();

                let _ = std::process::Command::new("cmd")
                    .args(&["/C", "del /f /q %localappdata%\\IconCache.db"])
                    .status();

                // 3. Restart Explorer (IMPORTANT: no "start", no cmd wrapper)
                let _ = std::process::Command::new("explorer.exe").status();

                logger::info("Explorer restart sequence completed");

                RUNNING.store(false, Ordering::SeqCst);
            })
            .unwrap();

        hotkey_manager.event_loop();
    });
}
