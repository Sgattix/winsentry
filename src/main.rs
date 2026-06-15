pub mod configuration;
pub mod modules;
pub mod logger;

use std::io::stdin;
use crate::modules::{debloater_watcher::debloater_worker, shortcut_manager::start_shortcut_worker};

struct SetupResult {
    config: configuration::WinSentryConfiguration,
}

fn setup() -> SetupResult {
    println!(
        "
    ┌──────┬──────┐
    │      │      │    _       ___      _____            __
    │      │      │   | |     / (_)___ / ___/___  ____  / /________  __
    ├──────┼──────┤   | | /| / / / __ \\\\__ \\/ _ \\/ __ \\/ __/ ___/ / / /
    ├──────┼──────┤   | |/ |/ / / / / /__/ /  __/ / / / /_/ /  / /_/ /
    │      │      │   |__/|__/_/_/ /_/____/\\___/_/ /_/\\__/_/   \\__, /
    │      │      │                                           /____/
    └──────┴──────┘
    "
    );

    let current_config = configuration::load_configuration().unwrap_or_default();

    if !current_config.user_scan_directories.is_empty() {
        println!("Configuration already exists. Loading configuration...");
        println!("Configuration loaded: {:?}", current_config);
        return SetupResult { config: current_config };
    }

    print!("Installing WinSentry...\n");
    println!("Please enter the desired key to pair with Ctrl+Alt+ for rebooting Windows Explorer (default: W):");
    let mut hotkey = String::new();
    stdin().read_line(&mut hotkey).unwrap();
    let mut hotkey = hotkey.trim().to_string();
    if hotkey.is_empty() { hotkey = "W".to_string(); }

    println!("Please enter any additional directories to scan for developer bloatware (e.g., C:\\Projects, D:\\Code), or press Enter to skip:");
    let mut user_input_dirs = String::new();
    stdin().read_line(&mut user_input_dirs).unwrap();
    
    let user_scan_directories: Vec<String> = user_input_dirs
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let new_config = configuration::WinSentryConfiguration {
        version: "1.0".into(),
        win_exp_reboot_hotkey: hotkey,
        user_scan_directories,
    };

    let _ = configuration::save_configuration(&new_config);

    SetupResult { config: new_config }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let config = configuration::load_configuration().unwrap_or_default();
        match args[1].as_str() {
            "--scan" => {
                let mut targets = std::collections::HashSet::new();
                targets.insert("node_modules".to_string());
                targets.insert("dist".to_string());
                modules::safe_queue::run_discovery_scan(config.user_scan_directories, targets, false);
                return;
            }
            "--purge" => {
                modules::safe_queue::interactive_purge();
                return;
            }
            _ => {}
        }
    }

    if !is_elevated::is_elevated() {
        println!("Please run WinSentry with administrator privileges to ensure all features work correctly.");
        let _ = std::process::Command::new("cmd").args(&["/C", "pause"]).status();
        return;
    }

    let setup_result = setup();

    println!(
        "WinSentry will run in the background in 5 seconds... To reboot Windows Explorer, press Ctrl+Alt+{}",
        setup_result.config.win_exp_reboot_hotkey
    );
    std::thread::sleep(std::time::Duration::from_secs(5));

    debloater_worker();
    
    start_shortcut_worker(setup_result.config);
}
