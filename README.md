Following guidelines at: https://stardance.hackclub.com/guides/great_readme

# WinSentry

A lightweight, native Rust system daemon that restores Windows usability by pairing an instant 300ms UI recovery watchdog with a dual-strategy storage manager that automatically purges OS telemetry while staging developer bloat for safe, user-controlled deletion.

> NOTE: WinSentry requires the latest [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) version to run. Please ensure you have it installed before using the application. Plus, you need to execute the program and Administrator privileges to run it. You may also need to allow WinSentry through Windows Defender or your antivirus software to ensure it can function properly.

## If you run into issues with the application, please check the [troubleshooting guide](https://github.com/Sgattix/winsentry/wiki/Troubleshooting) for common problems and solutions. If you still need assistance, feel free to open an issue on the GitHub repository.

## Showcase (showcase.mp4)

<video src="showcase.mp4" width="1000" height="540" controls></video>

## Try it

Download the latest release from the [releases page](https://github.com/Sgattix/WinSentry/releases)

## Quick Start Guide

1. Download the latest release from the [releases page](https://github.com/Sgattix/WinSentry/releases)
2. Extract the contents of the ZIP file to a location of your choice (e.g., `C:\WinSentry`)
3. Run `WinSentry.exe` to start the application (you may need to allow it through Windows Defender or your antivirus software)
4. Follow the on-screen instructions to set up the application and configure your preferences
5. Once set up, WinSentry will run in the background, monitoring your system and keeping it responsive while managing storage

## Features

- **UI Recovery Panic Button**: Instantly refreshes the taskbar and reboots the Windows Explorer process within 300ms after pressing the hotkey, restoring system responsiveness without requiring a full reboot. (Works even when the taskbar is frozen or unresponsive!) -> Defaults to Ctrl+Alt+W, but can be customized during setup or by editing the configuration file at %appdata%/WinSentry/Config.toml.
- **Storage Management**: Automatically deletes telemetry files and stages developer bloatware for user-controlled deletion, helping to free up disk space and improve system performance. (Runs scans in the background every 30 minutes in known directories like `%temp%`.)
- **Developer Bloat Detection**: Scans specified directories for common developer bloatware (e.g., `node_modules`, `target`, `bin/Debug/net*`) and stages them for review and deletion, allowing users to easily manage their storage without risking accidental deletion of important files. -> Defaults to Ctrl+Alt+Shift+C, but can be customized during setup or by editing the configuration file at %appdata%/WinSentry/Config.toml.
- **Custom Developer Bloat Paths**: Users can specify additional directories to scan for bloatware during setup or by editing the configuration file, allowing for personalized storage management based on individual development environments.
- **Lightweight and Native**: Built in Rust for optimal performance and minimal resource usage.

## How it Works

WinSentry consists of two main components: the UI Recovery Panic Button and the Storage Management system. The UI Recovery Panic Button monitors the system for signs of unresponsiveness and allows users to quickly refresh the taskbar and reboot the Windows Explorer process with a customizable hotkey. The Storage Management system continuously monitors specific directories for telemetry files and developer bloatware, automatically deleting telemetry files and staging bloatware for user-controlled deletion. This dual-strategy approach ensures that users can maintain a responsive system while also managing storage effectively.

## Credits and Acknowledgements

- [Sgattix](https://github.com/Sgattix)
