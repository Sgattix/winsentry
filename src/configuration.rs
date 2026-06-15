use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WinSentryConfiguration {
    pub version: String,
    pub win_exp_reboot_hotkey: String,
    pub user_scan_directories: Vec<String>,
}

impl Default for WinSentryConfiguration {
    fn default() -> Self {
        WinSentryConfiguration {
            version: "1.0".to_string(),
            win_exp_reboot_hotkey: "W".to_string(),
            user_scan_directories: Vec::new(),
        }
    }
}

pub fn load_configuration() -> Result<WinSentryConfiguration, confy::ConfyError> {
    let config: WinSentryConfiguration = confy::load("WinSentry", None)?;
    
    let mut sanitized = config.clone();
    sanitized.user_scan_directories = config.user_scan_directories
        .into_iter()
        .map(|mut p| {
            p = p.replace("/", "\\").replace("\\\\", "\\");
            while p.ends_with('\\') { p.pop(); }
            p
        })
        .collect();
    Ok(sanitized)
}

pub fn save_configuration(config: &WinSentryConfiguration) -> Result<(), confy::ConfyError> {
    confy::store("WinSentry", None, config)
}

pub fn get_win_exp_reboot_hotkey(config: &WinSentryConfiguration) -> Option<String> {
    Some(config.win_exp_reboot_hotkey.clone())
}

pub fn get_user_scan_directories(config: &WinSentryConfiguration) -> Option<Vec<String>> {
    Some(config.user_scan_directories.clone())
}
