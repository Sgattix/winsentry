use std::str::FromStr;
use std::collections::HashSet;
use std::sync::Arc;
use ::win_hotkeys::HotkeyManager;
use win_hotkeys::VKey;

use crate::logger;
use crate::configuration::WinSentryConfiguration;
use crate::modules::safe_queue::run_discovery_scan;
use crate::modules::taskbar_refresh::taskbar_refresh;

// Rimosso lo std::thread::spawn interno. Questa funzione ora blocca il thread in cui viene chiamata.
pub fn start_shortcut_worker(initial_config: WinSentryConfiguration) {
    logger::info("Avvio dello Shortcut Manager sul thread principale (Release Protected)...");

    let shared_config = Arc::new(initial_config);
    let config_for_hotkey_1 = Arc::clone(&shared_config);
    let config_for_hotkey_2 = Arc::clone(&shared_config);

    let mut hotkey_manager = HotkeyManager::new();

    let win_exp_key_str = config_for_hotkey_1.win_exp_reboot_hotkey.clone();
    if let Ok(vkey) = VKey::from_str(&win_exp_key_str) {
        hotkey_manager
            .register_hotkey(vkey, &[VKey::Control, VKey::Menu], move || {
                logger::info("Scorciatoia intercettata: riavvio shell Explorer...");
                std::thread::spawn(move || {
                    taskbar_refresh();
                });
            })
            .unwrap();
    }

    hotkey_manager
        .register_hotkey(VKey::C, &[VKey::Control, VKey::Menu, VKey::Shift], move || {
            logger::info("Scorciatoia intercettata: avvio scansione di sicurezza...");

            let mut targets = HashSet::new();
            targets.insert("node_modules".to_string());
            targets.insert("dist".to_string());
            targets.insert("__pycache__".to_string());
            targets.insert(".next".to_string());
            targets.insert("target".to_string());

            let config_local = Arc::clone(&config_for_hotkey_2);

            std::thread::spawn(move || {
                let user_directories = config_local.user_scan_directories.clone();
                logger::info(
                    &format!("Analisi in corso sulle directory caricate: {:?}", user_directories)
                );

                if user_directories.is_empty() {
                    logger::warn(
                        "Nessun percorso configurato trovato in memoria. Scansione annullata."
                    );
                    return;
                }

                run_discovery_scan(user_directories, targets, true);
            });
        })
        .unwrap();

    hotkey_manager.event_loop();
}
