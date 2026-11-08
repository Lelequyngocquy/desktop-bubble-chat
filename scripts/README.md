# Platform Automation Scripts

This folder contains platform-specific automation scripts to run, monitor, build, and diagnose **DesktopBubbleChat** across Windows, Linux, and macOS.

---

## Folder Structure

```
scripts/
├── tauri.be-logs.json       # Common Tauri dev config
├── windows/                 # PowerShell scripts (.ps1)
│   ├── start.ps1            # Launch application (dev, prod, web)
│   ├── status.ps1           # Resource & process monitor
│   ├── stop.ps1             # Terminate all processes & free port 1420
│   ├── build.ps1            # Production build pipeline
│   ├── check_env.ps1        # Toolchain & prerequisite diagnostics
│   ├── versioning.ps1       # Synchronize version across config files
│   ├── set-logo.ps1         # App icon & branding generator
│   └── appium-server.ps1    # WinAppDriver test server
├── linux/                   # Linux Bash scripts (.sh)
│   ├── start.sh             # Launch application (dev, prod, web)
│   ├── status.sh            # Process table & RAM breakdown
│   ├── stop.sh              # Terminate app & WebKit helper processes
│   ├── build.sh             # Production build pipeline
│   └── check_env.sh         # Linux GTK3, WebKitGTK & toolchain diagnostics
└── macos/                   # macOS Bash scripts (.sh)
    ├── start.sh             # Launch application (dev, prod, web)
    ├── status.sh            # Process table & RAM breakdown
    ├── stop.sh              # Terminate app & WebKit processes
    ├── build.sh             # Production build & .app bundle pipeline
    └── check_env.sh         # Xcode & toolchain diagnostics
```

---

## 1. Windows (`scripts/windows/`)

```powershell
# Development Mode (Hot-reload React + Rust backend logs)
.\scripts\windows\start.ps1

# Monitor running processes & RAM
.\scripts\windows\status.ps1

# Stop all processes & free port 1420
.\scripts\windows\stop.ps1

# Production Build
.\scripts\windows\build.ps1
```

---

## 2. Linux (`scripts/linux/`)

```bash
# Development Mode (Hot-reload React + Rust backend logs)
bash scripts/linux/start.sh

# Monitor running processes & RAM
bash scripts/linux/status.sh

# Stop all processes & free port 1420
bash scripts/linux/stop.sh

# Production Build
bash scripts/linux/build.sh
```

---

## 3. macOS (`scripts/macos/`)

```bash
# Development Mode (Hot-reload React + Rust backend logs)
bash scripts/macos/start.sh

# Monitor running processes & RAM
bash scripts/macos/status.sh

# Stop all processes & free port 1420
bash scripts/macos/stop.sh

# Production Build
bash scripts/macos/build.sh
```
