Following guidelines at: https://stardance.hackclub.com/guides/great_readme

# WinSentry

A lightweight, native Rust system daemon that restores Windows usability by pairing an instant 300ms UI recovery watchdog with a dual-strategy storage manager that automatically purges OS telemetry while staging developer bloat for safe, user-controlled deletion.

## Showcase

<video src="./showcase.mp4" width="1000" height="540" controls></video>

## Try it

Download the latest release from the [releases page](https://github.com/Sgattix/WinSentry/releases)

## Quick Start Guide

1. Download the latest release from the [releases page](https://github.com/Sgattix/WinSentry/releases)
2. Extract the contents of the ZIP file to a location of your choice (e.g., `C:\WinSentry`)
3. Run `WinSentry.exe` to start the application (you may need to allow it through Windows Defender or your antivirus software)
4. Follow the on-screen instructions to set up the application and configure your preferences
5. Once set up, WinSentry will run in the background, monitoring your system and keeping it responsive while managing storage

## Features

- **UI Recovery Panic Button**: Instantly refreshes the taskbar and reboots the Windows Explorer process within 300ms after pressing the hotkey, restoring system responsiveness without requiring a full reboot.
- **Storage Management**: Automatically deletes telemetry files and stages developer bloatware for user-controlled deletion, helping to free up disk space and improve system performance.
- **Customizable Hotkeys**: Allows users to set custom hotkeys for manually refreshing the taskbar and rebooting the Windows Explorer process.
- **Lightweight and Native**: Built in Rust for optimal performance and minimal resource usage.

## How it Works

WinSentry consists of two main components: the UI Recovery Panic Button and the Storage Management system. The UI Recovery Panic Button monitors the system for signs of unresponsiveness and allows users to quickly refresh the taskbar and reboot the Windows Explorer process with a customizable hotkey. The Storage Management system continuously monitors specific directories for telemetry files and developer bloatware, automatically deleting telemetry files and staging bloatware for user-controlled deletion. This dual-strategy approach ensures that users can maintain a responsive system while also managing storage effectively.

## Credits and Acknowledgements

- [Sgattix](https://github.com/Sgattix)
