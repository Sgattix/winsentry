pub mod configuration;
pub mod modules;
pub mod logger;

use std::{io::stdin};

use crate::{logger::create_log_file, modules::{debloater_watcher::debloater_worker, taskbar_refresh::taskbar_refresh}};


fn setup() -> String {
    print!("\x1B[2J\x1B[1;1H");



    println!("
┌──────┬──────┐
│      │      │    _       ___      _____            __
│      │      │   | |     / (_)___ / ___/___  ____  / /________  __
├──────┼──────┤   | | /| / / / __ \\\\__ \\/ _ \\/ __ \\/ __/ ___/ / / /
├──────┼──────┤   | |/ |/ / / / / /__/ /  __/ / / / /_/ /  / /_/ /
│      │      │   |__/|__/_/_/ /_/____/\\___/_/ /_/\\__/_/   \\__, /
│      │      │                                           /____/
└──────┴──────┘
    ");
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

    print!("Installing WinSentry...\n");

    println!(
        "Please enter the desired key to pair with Ctrl+Alt+ for rebooting Windows Explorer (default: Ctrl+Alt+W):"
    );
    let mut hotkey = String::new();
    stdin().read_line(&mut hotkey).unwrap();
    hotkey = hotkey.trim().into();
    
    if let Err(e) = configuration::create_configuration(
        installation_path.clone(),
        configuration::WinSentryConfiguration {
            version: "1.0".into(),
            win_exp_reboot_hotkey: hotkey,
        },
    ) {
        eprintln!("Error creating configuration: {}", e);
    }
    create_log_file(installation_path.clone()).unwrap_or_else(|e| {
        eprintln!("Error creating log file: {}", e);
    });
    return String::new();
}

fn main() {
        // Request UAC elevation on Windows startup to ensure the application has the necessary permissions to perform its functions, and to prevent issues with file deletion and hotkey registration
    if !is_elevated::is_elevated() {
        println!("Please run WinSentry with administrator privileges to ensure all features work correctly.");
        std::process::Command::new("cmd")
            .args(&["/C", "pause"])
            .spawn()
            .expect("Failed to request UAC elevation");
        return;
    }
    let installation_path = setup();
    let config_file_path = format!("{}\\config\\Config.toml", installation_path);

    // Run background service to listen for hotkey presses and perform actions
    println!("WinSentry will run in the background in 5 seconds... To reboot Windows Explorer, press Ctrl+Alt+{} (configurable in config file)", configuration::get_win_exp_reboot_hotkey(&config_file_path).unwrap_or("W".to_string()));
    std::thread::sleep(std::time::Duration::from_secs(5));
    // Hide console window and run in background
    #[cfg(target_os = "windows")]
    {
        use winapi::um::wincon::FreeConsole;
        unsafe {
            FreeConsole();
        }
    }
    debloater_worker();
    taskbar_refresh(config_file_path.clone());

    std::thread::park();
}
