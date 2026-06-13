use std::io::Read;

pub struct WinSentryConfiguration {
    pub version: String,
    pub win_exp_reboot_hotkey: String,
}

impl std::fmt::Debug for WinSentryConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WinSentryConfiguration")
            .field("version", &self.version)
            .field("win_exp_reboot_hotkey", &self.win_exp_reboot_hotkey)
            .finish()
    }
}

pub fn create_configuration(
    mut config_path: String,
    config: WinSentryConfiguration,
) -> Result<WinSentryConfiguration, std::io::Error> {
    let default_config_path: String = String::from(&format!(
        "{}\\WinSentry\\config",
        std::env::current_dir()?.to_str().unwrap()
    ));
    if config_path.is_empty() {
        config_path = default_config_path;
    }
    let config_file_path: &str = &format!("{}\\Config.toml", config_path);

    if !std::path::Path::new(&config_path).exists() {
        println!("Creating configuration directory at {}...", config_path);
        std::fs::create_dir_all(&config_path)?;
    }

    if !std::path::Path::new(&config_file_path).exists() {
        println!("Creating configuration file at {}...", config_file_path);
        std::fs::File::create(&config_file_path)?;
    }
    let _file = std::fs::File::open(&config_file_path)?;
    let config_content = format!(
        "version = \"{}\"\nwin_exp_reboot_hotkey = \"{}\"",
        config.version, config.win_exp_reboot_hotkey
    );
    std::fs::write(&config_file_path, config_content)?;
    return load_configuration(&config_file_path);
}

pub fn load_configuration(
    config_file_path: &str,
) -> Result<WinSentryConfiguration, std::io::Error> {
    let mut file = std::fs::File::open(config_file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = WinSentryConfiguration {
        version: contents
            .lines()
            .find(|line| line.starts_with("version"))
            .unwrap_or("version = \"1.0\"")
            .split('=')
            .nth(1)
            .unwrap_or("\"1.0\"")
            .trim()
            .to_string(),
        win_exp_reboot_hotkey: contents
            .lines()
            .find(|line| line.starts_with("win_exp_reboot_hotkey"))
            .unwrap_or("win_exp_reboot_hotkey = \"W\"")
            .split('=')
            .nth(1)
            .unwrap_or("\"W\"")
            .trim()
            .to_string(),
    };
    println!("Configuration loaded: {:?}", config);
    Ok(config)
}

pub fn config_exists(config_file_path: &str) -> bool {
    std::path::Path::new(&config_file_path).exists()
}

pub fn get_win_exp_reboot_hotkey(config_file_path: &str) -> Result<String, std::io::Error> {
    let config = load_configuration(config_file_path)?;
    Ok(config.win_exp_reboot_hotkey)
}
