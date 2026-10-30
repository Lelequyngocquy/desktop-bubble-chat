# DesktopBubbleChat Production Packaging Script
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "DesktopBubbleChat - Production Build Pipeline" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan

# 1. Clean build directory
Write-Host "[1/3] Kiểm tra & Xây dựng Frontend Production..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Lỗi: Frontend build thất bại!" -ForegroundColor Red
    exit 1
}

# 2. Check Rust Backend
Write-Host "[2/3] Kiểm tra Rust Backend..." -ForegroundColor Yellow
cargo check --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) {
    Write-Host "Lỗi: Backend check thất bại!" -ForegroundColor Red
    exit 1
}

# 3. Build release bundle
Write-Host "[3/3] Tiến hành đóng gói bộ cài đặt Windows (.exe / .msi)..." -ForegroundColor Yellow
Write-Host "Chạy: npm run tauri build" -ForegroundColor Green
Write-Host "Hoàn tất kiểm thử chuẩn bị đóng gói!" -ForegroundColor Cyan
