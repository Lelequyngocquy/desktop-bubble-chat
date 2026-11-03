<#
.SYNOPSIS
    Cập nhật đồng bộ số phiên bản ứng dụng vào cả 3 file (package.json, tauri.conf.json, Cargo.toml).

.DESCRIPTION
    Script này cho phép truyền tham số phiên bản (ví dụ: v0.1.1 hoặc 0.1.1) hoặc nhập trực tiếp khi chạy.
    Tự động chuẩn hóa định dạng, cập nhật 3 file cấu hình và in ra hướng dẫn Git Tag tương ứng.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/versioning.ps1 -Version v0.1.1
    powershell -ExecutionPolicy Bypass -File scripts/versioning.ps1 0.2.0
    powershell -ExecutionPolicy Bypass -File scripts/versioning.ps1
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0, Mandatory = $false)]
    [string]$Version
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding  = [System.Text.Encoding]::UTF8

# Định vị thư mục gốc dự án
$RootDir = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path "$RootDir\package.json")) {
    $RootDir = Get-Location
}

$PackageJsonPath = Join-Path $RootDir "package.json"
$TauriConfPath   = Join-Path $RootDir "src-tauri\tauri.conf.json"
$CargoTomlPath   = Join-Path $RootDir "src-tauri\Cargo.toml"

# Kiểm tra sự tồn tại của 3 file
$missingFiles = @()
if (-not (Test-Path $PackageJsonPath)) { $missingFiles += "package.json" }
if (-not (Test-Path $TauriConfPath))   { $missingFiles += "src-tauri\tauri.conf.json" }
if (-not (Test-Path $CargoTomlPath))   { $missingFiles += "src-tauri\Cargo.toml" }

if ($missingFiles.Count -gt 0) {
    Write-Host "[ERROR] Không tìm thấy các file sau: $($missingFiles -join ', ')" -ForegroundColor Red
    exit 1
}

# Đọc phiên bản hiện tại từ package.json
$currentVersion = "unknown"
if ((Get-Content $PackageJsonPath -Raw) -match '"version":\s*"([^"]+)"') {
    $currentVersion = $matches[1]
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   DESKTOP BUBBLE CHAT - TIỆN ÍCH CẬP NHẬT PHIÊN BẢN      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "Phiên bản hiện tại: " -NoNewline
Write-Host "v$currentVersion" -ForegroundColor Yellow

# Nếu chưa truyền tham số Version thì yêu cầu người dùng nhập
if ([string]::IsNullOrWhiteSpace($Version)) {
    Write-Host ""
    $Version = Read-Host "Nhập số phiên bản mới (ví dụ: v0.1.1 hoặc 0.1.1)"
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    Write-Host "[ABORT] Không có phiên bản nào được nhập. Đã hủy bỏ thao tác." -ForegroundColor Yellow
    exit 0
}

# Chuẩn hóa: loại bỏ chữ 'v' hoặc 'V' ở đầu nếu người dùng nhập 'v0.1.1'
$cleanVersion = $Version.Trim()
if ($cleanVersion -match '^[vV](.+)$') {
    $cleanVersion = $matches[1]
}

# Kiểm tra tính hợp lệ của SemVer (X.Y.Z hoặc X.Y.Z-tag)
if ($cleanVersion -notmatch '^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$') {
    Write-Host "[ERROR] Phiên bản '$Version' không hợp lệ! Vui lòng nhập đúng định dạng SemVer (ví dụ: 0.1.1, v0.2.0, 1.0.0-beta.1)" -ForegroundColor Red
    exit 1
}

$tagVersion = "v$cleanVersion"

Write-Host ""
Write-Host "-> Chuẩn bị cập nhật phiên bản: v$currentVersion => $tagVersion" -ForegroundColor Green
Write-Host ""

# Đối tượng mã hóa UTF-8 KHÔNG kèm BOM (ngăn lỗi serde_json "expected value at line 1 column 1")
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

# 1. Cập nhật package.json
try {
    $pkgContent = Get-Content $PackageJsonPath -Raw -Encoding UTF8
    $replacement = '${1}' + $cleanVersion + '${2}'
    $newPkgContent = $pkgContent -replace '(?m)^(\s*"version":\s*")[^"]+(")', $replacement
    [System.IO.File]::WriteAllText($PackageJsonPath, $newPkgContent, $utf8NoBom)
    Write-Host "  [OK] package.json              -> $cleanVersion" -ForegroundColor Green
} catch {
    Write-Host "  [FAIL] package.json: $_" -ForegroundColor Red
    exit 1
}

# 2. Cập nhật src-tauri/tauri.conf.json
try {
    $tauriContent = Get-Content $TauriConfPath -Raw -Encoding UTF8
    $replacement = '${1}' + $cleanVersion + '${2}'
    $newTauriContent = $tauriContent -replace '(?m)^(\s*"version":\s*")[^"]+(")', $replacement
    [System.IO.File]::WriteAllText($TauriConfPath, $newTauriContent, $utf8NoBom)
    Write-Host "  [OK] src-tauri/tauri.conf.json -> $cleanVersion" -ForegroundColor Green
} catch {
    Write-Host "  [FAIL] src-tauri/tauri.conf.json: $_" -ForegroundColor Red
    exit 1
}

# 3. Cập nhật src-tauri/Cargo.toml (Chỉ thay thế version ở đầu dòng thuộc [package])
try {
    $cargoContent = Get-Content $CargoTomlPath -Raw -Encoding UTF8
    $cargoReplacement = 'version = "' + $cleanVersion + '"'
    $newCargoContent = $cargoContent -replace '(?m)^version\s*=\s*"[^"]+"', $cargoReplacement
    [System.IO.File]::WriteAllText($CargoTomlPath, $newCargoContent, $utf8NoBom)
    Write-Host "  [OK] src-tauri/Cargo.toml      -> $cleanVersion" -ForegroundColor Green
} catch {
    Write-Host "  [FAIL] src-tauri/Cargo.toml: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " CẬP NHẬT THÀNH CÔNG CẢ 3 FILE LÊN PHIÊN BẢN $tagVersion ! " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Các bước tiếp theo để phát hành lên GitHub Releases:" -ForegroundColor White
Write-Host "  1. Kiểm tra build:  " -NoNewline
Write-Host "npm run build" -ForegroundColor Yellow
Write-Host "  2. Commit thay đổi: " -NoNewline
$commitText = "git add . ; git commit -m 'chore: release $tagVersion'"
Write-Host $commitText -ForegroundColor Yellow
Write-Host "  3. Tạo Git Tag:     " -NoNewline
Write-Host "git tag $tagVersion" -ForegroundColor Yellow
Write-Host "  4. Đẩy lên GitHub:  " -NoNewline
Write-Host "git push origin master --tags" -ForegroundColor Yellow
Write-Host ""
