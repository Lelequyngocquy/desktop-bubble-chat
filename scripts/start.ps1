<#
.SYNOPSIS
    Starts the DesktopBubbleChat application (Dev, Prod, or Web mode).
.DESCRIPTION
    Checks and frees port 1420 first, configures environment variables (Cargo/Rust PATH),
    verifies prerequisites, and runs the application in the specified mode.
.PARAMETER Mode
    'dev'  (default): Run Tauri in development mode (hot-reload React + Rust)
    'prod': Run the built release binary (DesktopBubbleChat.exe)
    'web':  Run only the Vite dev server (browser preview at http://localhost:1420)
#>
[CmdletBinding()]
param (
    [ValidateSet("dev", "prod", "web")]
    [string]$Mode = "dev"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "   DesktopBubbleChat - Application Launcher         " -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "[INFO] Project root: $ProjectRoot" -ForegroundColor Gray
Write-Host "[INFO] Target mode : $Mode" -ForegroundColor Yellow

# 1. Pre-launch Check: Verify and clean up port 1420 if occupied
if ($Mode -in @("dev", "web")) {
    Write-Host "[INFO] Checking port 1420 availability..." -ForegroundColor Gray
    try {
        $Port1420 = Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue
        if ($Port1420) {
            $Pids = $Port1420 | Select-Object -ExpandProperty OwningProcess -Unique
            foreach ($p in $Pids) {
                Write-Host "[WARN] Port 1420 is currently occupied by PID $p. Terminating process..." -ForegroundColor Yellow
                Stop-Process -Id $p -Force -ErrorAction SilentlyContinue
            }
            Start-Sleep -Milliseconds 600
            Write-Host "[SUCCESS] Port 1420 is now free." -ForegroundColor Green
        } else {
            Write-Host "[INFO] Port 1420 is clean and available." -ForegroundColor Green
        }
    } catch {
        Write-Host "[INFO] Port 1420 status verified." -ForegroundColor Gray
    }
}

# 2. Configure PATH for Cargo and Rust
$CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $CargoBin) {
    if ($env:PATH -notlike "*$CargoBin*") {
        $env:PATH = "$CargoBin;$env:PATH"
        Write-Host "[INFO] Added Cargo to PATH: $CargoBin" -ForegroundColor Green
    }
} else {
    Write-Host "[WARN] Cargo directory not found at $CargoBin" -ForegroundColor Yellow
}

# 3. Web mode: Frontend preview in browser
if ($Mode -eq "web") {
    Write-Host ""
    Write-Host "[INFO] Starting Vite Web Dev Server..." -ForegroundColor Green
    Set-Location $ProjectRoot
    npm run dev
    exit 0
}

# 4. Prod mode: Run built release executable
if ($Mode -eq "prod") {
    $ReleaseExe = Join-Path $ProjectRoot "src-tauri\target\release\DesktopBubbleChat.exe"
    if (Test-Path $ReleaseExe) {
        Write-Host ""
        Write-Host "[INFO] Launching Release executable: $ReleaseExe" -ForegroundColor Green
        Start-Process -FilePath $ReleaseExe
        Write-Host "[SUCCESS] Application started successfully." -ForegroundColor Green
        exit 0
    } else {
        Write-Host ""
        Write-Host "[ERROR] Release executable not found at: $ReleaseExe" -ForegroundColor Red
        Write-Host "[INFO] Please run 'scripts/build.ps1' first to package the application." -ForegroundColor Yellow
        exit 1
    }
}

# 5. Dev mode: Full Tauri desktop dev server
if ($Mode -eq "dev") {
    # Check MSVC Linker
    $HasLinker = Get-Command "link.exe" -ErrorAction SilentlyContinue
    if (-not $HasLinker) {
        $VsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
        if (Test-Path $VsWhere) {
            $VsInstallPath = & $VsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
            if ($VsInstallPath) {
                $LinkerPath = & $VsWhere -latest -find **/link.exe
                if ($LinkerPath) {
                    $LinkerDir = Split-Path -Parent $LinkerPath[0]
                    $env:PATH = "$LinkerDir;$env:PATH"
                    Write-Host "[INFO] Found MSVC linker: $LinkerDir" -ForegroundColor Green
                }
            }
        }
    }

    Write-Host ""
    Write-Host "[INFO] Starting Tauri Dev (Vite + Tauri hot-reload)..." -ForegroundColor Green
    Write-Host "[INFO] Press Ctrl+C to terminate the dev server." -ForegroundColor Gray
    Set-Location $ProjectRoot
    npm run tauri dev
}
