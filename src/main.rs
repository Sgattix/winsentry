pub mod configuration;
use ::win_hotkeys::HotkeyManager;
use std::{io::stdin, str::FromStr};
use win_hotkeys::VKey;

use crate::configuration::get_win_exp_reboot_hotkey;

fn setup() -> String {
    println!("Welcome to WinSentry!");
    println!("Please enter the installation path for WinSentry (default: current directory):");
    let mut installation_path = String::new();
    stdin().read_line(&mut installation_path).unwrap();

    installation_path = installation_path.trim().into();
    if configuration::config_exists(&format!("{}\\Config.toml", installation_path)) {
        println!(
            "Configuration already exists at {}. Loading configuration...",
            installation_path
        );
        match configuration::load_configuration(&format!("{}\\Config.toml", installation_path)) {
            Ok(config) => println!("Configuration loaded: {:?}", config),
            Err(e) => eprintln!("Error loading configuration: {}", e),
        }
        return installation_path;
    }

    print!("Installing WinSentry to {}...\n", installation_path);

    println!(
        "Please enter the desired key to pair with Ctrl+Alt+ for rebooting Windows Explorer (default: Ctrl+Alt+W):"
    );
    let mut hotkey = String::new();
    stdin().read_line(&mut hotkey).unwrap();
    hotkey = hotkey.trim().into();

    if let Err(e) = configuration::create_configuration(
        installation_path,
        configuration::WinSentryConfiguration {
            version: "1.0".into(),
            win_exp_reboot_hotkey: hotkey,
        },
    ) {
        eprintln!("Error creating configuration: {}", e);
    }
    return String::new();
}

fn main() {
    let installation_path = setup();
    let config_file_path = format!("{}\\config\\Config.toml", installation_path);

    // Run background service to listen for hotkey presses and perform actions
    println!("WinSentry is running in the background. Press Ctrl+C to exit.");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut hotkey_manager = HotkeyManager::new();

        hotkey_manager
            .register_hotkey(
                VKey::from_str(
                    &get_win_exp_reboot_hotkey(&config_file_path).unwrap_or("W".to_string()),
                )
                .unwrap(),
                &[VKey::Control, VKey::Menu],
                || {
                    println!("Rebooting Windows Explorer...");
                    std::process::Command::new("cmd")
                        .args(&["/C", "taskkill /F /IM explorer.exe && start explorer.exe"])
                        .spawn()
                        .expect("Failed to reboot Windows Explorer");

                    // Clear windows icon cache to prevent icon issues after rebooting explorer
                    std::process::Command::new("cmd")
                        .args(&["/C", "ie4uinit.exe -ClearIconCache"])
                        .spawn()
                        .expect("Failed to clear icon cache");

                    std::process::Command::new("cmd")
                        .args(&["/C", "del /f /q %localappdata%\\IconCache.db"])
                        .spawn()
                        .expect("Failed to delete icon cache file");

                    // Rename registry key to force Windows to rebuild icon cache on next boot (Computer\HKEY_CURRENT_USER\Software\Classes\.png -> Computer\HKEY_CURRENT_USER\Software\Classes\.png_old)
                    std::process::Command::new("cmd")
                        .args(&[
                            "/C",
                            "reg export \"HKEY_CURRENT_USER\\Software\\Classes\\.png\" \"%temp%\\icon_cache_backup.reg\" && reg delete \"HKEY_CURRENT_USER\\Software\\Classes\\.png\" /f && reg add \"HKEY_CURRENT_USER\\Software\\Classes\\.png_old\" /f && reg import \"%temp%\\icon_cache_backup.reg\" && del /f /q \"%temp%\\icon_cache_backup.reg\"",
                        ])
                        .spawn()
                        .expect("Failed to rename registry key for icon cache");

                    // Print powershell command due to windows security features preventing the above commands from fully clearing the icon cache without a reboot, and the powershell command forces Windows to refresh all explorer windows which can help mitigate icon issues until the next reboot
                    println!("Due to Windows security features, the icon cache may not be fully cleared until the next reboot. To refresh all explorer windows and mitigate icon issues until then, you can run the following PowerShell command:\n\nGet-AppXPackage -AllUsers | Where-Object {{$_.InstallLocation -like \"*ShellExperienceHost*\"}} | Foreach {{Add-AppxPackage -DisableDevelopmentMode -Register \"$($_.InstallLocation)\\AppXManifest.xml\"}}\n");

                    // Reboot explorer again to ensure all changes take effect
                    std::process::Command::new("cmd")
                        .args(&["/C", "taskkill /F /IM explorer.exe && start explorer.exe"])
                        .spawn()
                        .expect("Failed to reboot Windows Explorer after clearing icon cache");
                    
                },
            )
            .unwrap();

        hotkey_manager.event_loop();
    }
}
