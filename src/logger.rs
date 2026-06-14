use std::{fs::OpenOptions, io::Write, path::Path, sync::OnceLock};

static LOG_FILE_PATH: OnceLock<String> = OnceLock::new();

pub fn create_log_file(mut installation_path: String) -> Result<(), std::io::Error> {
    let default_log_path: String = String::from(&format!(
        "{}\\WinSentry\\log",
        std::env::current_dir()?.to_str().unwrap()
    ));
    if installation_path.is_empty() {
        installation_path = default_log_path;
    }
    let log_file_path = format!(
        "{}\\log-{}.txt",
        installation_path,
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
    );

    let path = Path::new(&log_file_path);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            eprintln!("Failed to create log directory {}: {}", parent.display(), e);
            e
        })?;
    }

    if !path.exists() {
        let file = std::fs::File::create(&log_file_path).unwrap_or_else(|e| {
            eprintln!("Failed to create log file {}: {}", log_file_path, e);
            std::process::exit(1);
        });

        let initial_message = format!(
            "[INFO] [{}] Log file created at {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            log_file_path
        );

        let _ = writeln!(&file, "{}", initial_message);
    }

    let _ = LOG_FILE_PATH.set(log_file_path);

    Ok(())
}

fn write_to_file(line: &str) {
    if let Some(path) = LOG_FILE_PATH.get() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{}", line);
        }
    }
}

pub fn info(message: &str) {
    let line = format!(
        "[INFO] [{}] {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        message
    );

    eprintln!("{}", line);
    write_to_file(&line);
}

pub fn error(message: &str) {
    let line = format!(
        "[ERROR] [{}] {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        message
    );

    eprintln!("{}", line);
    write_to_file(&line);
}
