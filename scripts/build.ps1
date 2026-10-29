<#
.SYNOPSIS
    Automates the build and packaging process for DesktopBubbleChat.
.DESCRIPTION
    1. Verifies build environment (Node.js, Cargo, MSVC Linker).
    2. Builds the Frontend (TypeScript + Vite).
    3. Builds and packages the Tauri Release installer (.exe / .msi / NSIS).
    4. Reports generated installer artifacts and opens destination directory.
.PARAMETER OpenFolder
    Automatically opens the installer directory upon completion. Default is true.
.PARAMETER FrontendOnly
    Builds only the Frontend web assets without compiling the Rust backend.
#>
[CmdletBinding()]
param (
    [switch]$OpenFolder = $true,
    [switch]$FrontendOnly = $false
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "   DesktopBubbleChat - Release Builder              " -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "[INFO] Project root: $ProjectRoot" -ForegroundColor Gray

# 1. Configure Cargo PATH
$CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $CargoBin) {
    if ($env:PATH -notlike "*$CargoBin*") {
        $env:PATH = "$CargoBin;$env:PATH"
    }
    Write-Host "[INFO] Cargo toolchain: Loaded into PATH" -ForegroundColor Green
} else {
    Write-Host "[WARN] Cargo directory not found at $CargoBin" -ForegroundColor Yellow
}

# 2. Compile Frontend (TypeScript & Vite)
Write-Host ""
Write-Host "----------------------------------------------------" -ForegroundColor Gray
Write-Host "[STEP 1/2] Compiling Frontend (TypeScript + Vite)..." -ForegroundColor Yellow
Write-Host "----------------------------------------------------" -ForegroundColor Gray

Set-Location $ProjectRoot
$BuildStartTime = Get-Date

try {
    npm run build
    Write-Host "[SUCCESS] Frontend build completed." -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Frontend compilation failed: $_" -ForegroundColor Red
    exit 1
}

if ($FrontendOnly) {
    Write-Host ""
    Write-Host "[SUCCESS] FrontendOnly mode finished. Assets are ready in 'dist/'." -ForegroundColor Green
    exit 0
}

# 3. Check MSVC Linker before packaging
$HasLinker = Get-Command "link.exe" -ErrorAction SilentlyContinue
if (-not $HasLinker) {
    $VsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $VsWhere) {
        $LinkerPath = & $VsWhere -latest -find **/link.exe
        if ($LinkerPath) {
            $LinkerDir = Split-Path -Parent $LinkerPath[0]
            $env:PATH = "$LinkerDir;$env:PATH"
            Write-Host "[INFO] Found MSVC linker: $LinkerPath[0]" -ForegroundColor Green
        }
    }
}

# 4. Package Tauri Release
Write-Host ""
Write-Host "----------------------------------------------------" -ForegroundColor Gray
Write-Host "[STEP 2/2] Packaging Tauri Release Bundle..." -ForegroundColor Yellow
Write-Host "----------------------------------------------------" -ForegroundColor Gray

try {
    npm run tauri build
} catch {
    Write-Host ""
    Write-Host "[ERROR] Tauri packaging failed: $_" -ForegroundColor Red
    exit 1
}

$BuildDuration = [math]::Round(((Get-Date) - $BuildStartTime).TotalSeconds, 1)

# 5. Output build artifacts
$BundleDir = Join-Path $ProjectRoot "src-tauri\target\release\bundle"

Write-Host ""
Write-Host "====================================================" -ForegroundColor Green
Write-Host "   PACKAGING COMPLETED SUCCESSFULLY ($BuildDuration s)" -ForegroundColor Green
Write-Host "====================================================" -ForegroundColor Green

if (Test-Path $BundleDir) {
    Write-Host ""
    Write-Host "[INFO] Generated installer packages:" -ForegroundColor Cyan
    Get-ChildItem -Path $BundleDir -Recurse -Include "*.msi", "*.exe" | ForEach-Object {
        $SizeMB = [math]::Round($_.Length / 1MB, 2)
        Write-Host "   - $($_.Name) ($SizeMB MB) -> $($_.FullName)" -ForegroundColor White
    }

    if ($OpenFolder) {
        Write-Host ""
        Write-Host "[INFO] Opening output directory..." -ForegroundColor Yellow
        Start-Process "explorer.exe" -ArgumentList $BundleDir
    }
} else {
    $ReleaseExe = Join-Path $ProjectRoot "src-tauri\target\release\DesktopBubbleChat.exe"
    if (Test-Path $ReleaseExe) {
        Write-Host "[INFO] Output executable: $ReleaseExe" -ForegroundColor Cyan
    }
}
