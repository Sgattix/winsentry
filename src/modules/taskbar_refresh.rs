use std::path::PathBuf;
use std::fs;
use std::process::Command;
use std::os::windows::process::CommandExt;

use crate::{ logger };

// Import Win32 core handles
use windows_sys::Win32::Foundation::CloseHandle;

pub fn taskbar_refresh() {
    logger::info("Initiating structural Explorer refresh...");

    // Fetch explorer PIDs using a lightweight, targeted tasklist query
    if
        let Ok(output) = Command::new("tasklist")
            .args(&["/FI", "IMAGENAME eq explorer.exe", "/FO", "CSV", "/NH"])
            .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            // Parse CSV format: "explorer.exe","1234","Console","1",...
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() > 1 {
                let pid_str = parts[1].trim_matches('"');
                if let Ok(pid) = pid_str.parse::<u32>() {
                    unsafe {
                        let handle = windows_sys::Win32::System::Threading::OpenProcess(
                            windows_sys::Win32::System::Threading::PROCESS_TERMINATE,
                            0,
                            pid
                        );
                        if handle != 0 {
                            windows_sys::Win32::System::Threading::TerminateProcess(handle, 2);
                            CloseHandle(handle);
                        }
                    }
                }
            }
        }
    }

    // 2. Clear legacy cache natively using ie4uinit
    let _ = Command::new("ie4uinit.exe").arg("-ClearIconCache").status();

    // 3. Safely delete IconCache.db natively
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let mut cache_path = PathBuf::from(local_app_data);
        cache_path.push("IconCache.db");
        if cache_path.exists() {
            let _ = fs::remove_file(&cache_path);
        }
    }

    // 4. Handle Registry transformations natively via reg.exe
    // (HKCU modifications do not require administrator elevation)
    let temp_dir = std::env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());
    let backup_reg_path = format!("{}\\icon_cache_backup.reg", temp_dir);

    let _ = Command::new("reg")
        .args(&["export", "HKEY_CURRENT_USER\\Software\\Classes\\.png", &backup_reg_path, "/y"])
        .status();
    let _ = Command::new("reg")
        .args(&["delete", "HKEY_CURRENT_USER\\Software\\Classes\\.png", "/f"])
        .status();
    let _ = Command::new("reg")
        .args(&["add", "HKEY_CURRENT_USER\\Software\\Classes\\.png_old", "/f"])
        .status();
    let _ = Command::new("reg").args(&["import", &backup_reg_path]).status();
    let _ = fs::remove_file(&backup_reg_path);

    // 5. SECURELY INITIALIZE THE MASTER USER INTERFACE SHELL
    // Give the OS exactly 800ms to clean up the closed process handles
    std::thread::sleep(std::time::Duration::from_millis(800));

    // Use CREATE_NEW_CONSOLE (0x00000010) instead of DETACHED_PROCESS.
    // On Windows 11 and late Windows 10 updates, this tells the system kernel
    // to spawn the process completely outside of your tool's UI container.
    const CREATE_NEW_CONSOLE: u32 = 0x00000010;

    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let absolute_explorer = format!("{}\\{}", windir, "explorer.exe");

    let mut shell_cmd = Command::new(&absolute_explorer);
    shell_cmd
        .current_dir(&windir) // Locks the context directory to Windows root
        .creation_flags(CREATE_NEW_CONSOLE);

    match shell_cmd.spawn() {
        Ok(_) => logger::info("Master Explorer GUI successfully restored."),
        Err(e) => {
            logger::error(&format!("Standard launch dropped. Executing modern AppX bypass: {}", e));
            // Fallback: Uses Windows system powershell core routing to re-initialize shell environments
            let _ = Command::new("powershell")
                .args(
                    &[
                        "-NoProfile",
                        "-WindowStyle",
                        "Hidden",
                        "-Command",
                        "Start-Process explorer.exe",
                    ]
                )
                .creation_flags(CREATE_NEW_CONSOLE)
                .spawn();
        }
    }

    logger::info("Explorer refresh complete.");
}
