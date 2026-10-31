# DesktopBubbleChat Production Build & Packaging Script
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "DesktopBubbleChat - Production Build Pipeline" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan

# 1. Build Frontend Production Bundle
Write-Host "`n[1/3] Building Frontend (TypeScript & Vite)..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Frontend build failed!" -ForegroundColor Red
    exit 1
}

# 2. Check Rust Backend
Write-Host "`n[2/3] Verifying Rust Backend Codebase..." -ForegroundColor Yellow
cargo check --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Rust backend check failed!" -ForegroundColor Red
    exit 1
}

# 3. Build & Package Production Release with Tauri
Write-Host "`n[3/3] Packaging Production Release (.exe / installer bundles)..." -ForegroundColor Yellow
npx tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Tauri production build failed!" -ForegroundColor Red
    exit 1
}

# 4. Display Output Paths
Write-Host "`n=============================================" -ForegroundColor Green
Write-Host "PRODUCTION BUILD COMPLETED SUCCESSFULLY!" -ForegroundColor Green
Write-Host "=============================================" -ForegroundColor Green

$ReleaseDir = Join-Path $ProjectRoot "src-tauri\target\release"
$BundleDir = Join-Path $ReleaseDir "bundle"

Write-Host "`nArtifacts generated:" -ForegroundColor Cyan

# 4.1 Standalone Binary
$ExeFiles = Get-ChildItem -Path $ReleaseDir -Filter "*.exe" -File -Depth 0 -ErrorAction SilentlyContinue | Where-Object { $_.Name -notmatch "build_script" }
if ($ExeFiles) {
    Write-Host "`n[Executable Binary]" -ForegroundColor Yellow
    foreach ($exe in $ExeFiles) {
        $sizeMB = [math]::Round($exe.Length / 1MB, 2)
        Write-Host "  -> File: $($exe.Name) ($sizeMB MB)" -ForegroundColor White
        Write-Host "     Path: $($exe.FullName)" -ForegroundColor Green
    }
}

# 4.2 Installers (NSIS / MSI)
if (Test-Path $BundleDir) {
    $Installers = Get-ChildItem -Path $BundleDir -Recurse -Include *.exe, *.msi -File -ErrorAction SilentlyContinue
    if ($Installers) {
        Write-Host "`n[Installers / Distribution Bundles]" -ForegroundColor Yellow
        foreach ($item in $Installers) {
            $sizeMB = [math]::Round($item.Length / 1MB, 2)
            Write-Host "  -> Package: $($item.Name) ($sizeMB MB)" -ForegroundColor White
            Write-Host "     Path:    $($item.FullName)" -ForegroundColor Green
        }
    }
    Write-Host "`n[Output Folder]" -ForegroundColor Yellow
    Write-Host "  -> $BundleDir" -ForegroundColor Cyan
} else {
    Write-Host "`n[Output Folder]" -ForegroundColor Yellow
    Write-Host "  -> $ReleaseDir" -ForegroundColor Cyan
}

Write-Host "`nBuild finished at: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')`n" -ForegroundColor Gray
