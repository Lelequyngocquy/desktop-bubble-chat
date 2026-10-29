# Automation Scripts

This folder contains PowerShell scripts to develop, build, run, and diagnose **DesktopBubbleChat**.

---

## 1. Launch Application (`scripts/start.ps1`)

Automatically configures the Rust/Cargo PATH and launches the application in the specified mode.

```powershell
# Development Mode (Default): Runs React + Tauri dev server with hot-reloading
.\scripts\start.ps1
# or
.\scripts\start.ps1 -Mode dev
# or
npm run app:start

# Web Mode: Starts only the Vite dev server for browser preview
.\scripts\start.ps1 -Mode web

# Production Mode: Runs the compiled release executable
.\scripts\start.ps1 -Mode prod
```

---

## 2. Stop Processes (`scripts/stop.ps1`)

Terminates all running DesktopBubbleChat instances, floating chat heads, and dev servers on port 1420.

```powershell
.\scripts\stop.ps1
# or
npm run app:stop
```

---

## 3. Package Application (`scripts/build.ps1`)

Compiles the frontend and builds standalone Windows release installers (`.exe` / `.msi`).

```powershell
# Full packaging (Frontend + Tauri Release Bundle) and opens output directory
.\scripts\build.ps1
# or
npm run app:build

# Build only the frontend assets into 'dist/'
.\scripts\build.ps1 -FrontendOnly
```

---

## 4. Diagnostics (`scripts/check_env.ps1`)

Verifies the required toolchains (Node.js, npm, rustc, cargo, MSVC linker `link.exe`, and WebView2 Runtime).

```powershell
.\scripts\check_env.ps1
# or
npm run app:check
```
