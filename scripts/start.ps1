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

# 1. Pre-launch Check: Clean up running app instances, orphaned webviews, and port 1420
Write-Host "[INFO] Checking and cleaning stale application processes..." -ForegroundColor Gray
$TargetApps = @("DesktopBubbleChat", "desktop-bubble-chat")
foreach ($tName in $TargetApps) {
    $existingProcs = Get-Process -Name $tName -ErrorAction SilentlyContinue
    if ($existingProcs) {
        foreach ($p in $existingProcs) {
            Write-Host "[WARN] Found running instance $tName (PID: $($p.Id)). Terminating..." -ForegroundColor Yellow
            Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        }
    }
}

# Kill orphaned msedgewebview2 instances belonging to DesktopBubbleChat to release directory file locks
try {
    $wvProcs = Get-CimInstance Win32_Process -Filter "Name LIKE '%msedgewebview2%'" -ErrorAction SilentlyContinue |
        Where-Object { $_.CommandLine -like "*com.desktopbubblechat.app*" -or $_.CommandLine -like "*DesktopBubbleChat*" }
    if ($wvProcs) {
        foreach ($wv in $wvProcs) {
            Write-Host "[WARN] Terminating orphaned WebView2 process (PID: $($wv.ProcessId))..." -ForegroundColor Yellow
            Stop-Process -Id $wv.ProcessId -Force -ErrorAction SilentlyContinue
        }
    }
} catch {
    # Ignore CIM query errors
}

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
    npm run dev -- --force
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

# 5. Dev mode: Full Tauri desktop dev server với Backend Logs ưu tiên
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

    # Bật đầy đủ biến môi trường Log cho Rust Backend
    $env:RUST_LOG = "info,desktop_bubble_chat=debug,desktop_bubble_chat_lib=debug"
    $env:RUST_BACKTRACE = "1"

    # Tạo thư mục lưu log riêng cho Frontend
    $LogDir = Join-Path $ProjectRoot "logs"
    if (-not (Test-Path $LogDir)) {
        New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
    }
    $FeLogFile = Join-Path $LogDir "frontend.log"
    $FeErrFile = Join-Path $LogDir "frontend-error.log"

    Write-Host ""
    Write-Host "[INFO] Khoi chay Vite Dev Server ngam (Log FE duoc tach ra logs/frontend.log)..." -ForegroundColor Cyan

    # Khoi chay Vite Dev Server duoi tien trinh ngam, tach sach toan bo log FE khoi terminal
    $ViteProcess = Start-Process -FilePath "cmd.exe" `
        -ArgumentList "/c npm run dev" `
        -WorkingDirectory $ProjectRoot `
        -RedirectStandardOutput $FeLogFile `
        -RedirectStandardError $FeErrFile `
        -PassThru -NoNewWindow

    # Cho Vite Dev Server san sang va phan hoi HTTP 200 tren port 1420
    Write-Host "[INFO] Dang cho Vite Frontend Server san sang tren http://localhost:1420..." -ForegroundColor Gray
    $retry = 40
    $ready = $false
    while ($retry -gt 0) {
        try {
            $resp = Invoke-WebRequest -Uri "http://localhost:1420" -UseBasicParsing -TimeoutSec 1 -ErrorAction SilentlyContinue
            if ($resp -and $resp.StatusCode -eq 200) {
                $ready = $true
                break
            }
        } catch {}
        Start-Sleep -Milliseconds 250
        $retry--
    }

    if ($ready) {
        Write-Host "[SUCCESS] Vite Frontend Server da san sang tren http://localhost:1420 (HTTP 200)" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Timeout: Vite Frontend Server khong phan hoi tren http://localhost:1420!" -ForegroundColor Red
        if (Test-Path $FeErrFile) {
            Get-Content $FeErrFile -Tail 15 | ForEach-Object { Write-Host "[Vite Error] $_" -ForegroundColor Red }
        }
        if (Test-Path $FeLogFile) {
            Get-Content $FeLogFile -Tail 15 | ForEach-Object { Write-Host "[Vite Log] $_" -ForegroundColor Yellow }
        }
        exit 1
    }

    $BeConfigPath = Join-Path $ProjectRoot "scripts\tauri.be-logs.json"

    Write-Host ""
    Write-Host "====================================================" -ForegroundColor Green
    Write-Host "   STREAMING BACKEND LOGS (Rust / Tauri / OS)       " -ForegroundColor Green
    Write-Host "====================================================" -ForegroundColor Green
    Write-Host "[INFO] Terminal hien tai tap trung 100% vao LOG BACKEND (BE)." -ForegroundColor Yellow
    Write-Host "[INFO] Log FRONTEND (FE) duoc luu rieng tai: logs/frontend.log" -ForegroundColor Gray
    Write-Host "[INFO] Nhan Ctrl+C de dung toan bo ung dung." -ForegroundColor Gray
    Write-Host ""

    try {
        Set-Location $ProjectRoot
        npx tauri dev --no-dev-server-wait --config $BeConfigPath
    } finally {
        Write-Host "`n[INFO] Dang don dep tien trinh..." -ForegroundColor Yellow
        if ($ViteProcess -and -not $ViteProcess.HasExited) {
            Stop-Process -Id $ViteProcess.Id -Force -ErrorAction SilentlyContinue
        }
        $conn = Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue
        if ($conn) {
            $conn | Select-Object -ExpandProperty OwningProcess -Unique | ForEach-Object {
                Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue
            }
        }
        # Terminate any remaining app process and child webview
        Get-Process -Name "desktop-bubble-chat","DesktopBubbleChat" -ErrorAction SilentlyContinue |
            ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }
        try {
            Get-CimInstance Win32_Process -Filter "Name LIKE '%msedgewebview2%'" -ErrorAction SilentlyContinue |
                Where-Object { $_.CommandLine -like "*com.desktopbubblechat.app*" -or $_.CommandLine -like "*DesktopBubbleChat*" } |
                ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
        } catch {}
        Write-Host "[SUCCESS] Da dung toan bo tien trinh Dev." -ForegroundColor Green
    }
}
