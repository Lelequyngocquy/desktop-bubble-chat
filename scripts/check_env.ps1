<#
.SYNOPSIS
    Verifies development environment prerequisites for DesktopBubbleChat.
#>
[CmdletBinding()]
param ()

Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "   DesktopBubbleChat - Environment Diagnostics      " -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan

# Configure Cargo PATH
$CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $CargoBin) -and ($env:PATH -notlike "*$CargoBin*")) {
    $env:PATH = "$CargoBin;$env:PATH"
}

# 1. Node.js & npm
Write-Host ""
Write-Host "[1] Node.js & npm:" -ForegroundColor Yellow
$nodeCmd = Get-Command "node" -ErrorAction SilentlyContinue
if ($nodeCmd) {
    $nodeVer = & node --version
    Write-Host "   [OK] Node.js: $nodeVer" -ForegroundColor Green
} else {
    Write-Host "   [FAIL] Node.js is not installed!" -ForegroundColor Red
}

$npmCmd = Get-Command "npm" -ErrorAction SilentlyContinue
if ($npmCmd) {
    $npmVer = & npm --version
    Write-Host "   [OK] npm: v$npmVer" -ForegroundColor Green
} else {
    Write-Host "   [FAIL] npm is not installed!" -ForegroundColor Red
}

# 2. Rust & Cargo
Write-Host ""
Write-Host "[2] Rust Toolchain:" -ForegroundColor Yellow
$rustcCmd = Get-Command "rustc" -ErrorAction SilentlyContinue
if ($rustcCmd) {
    $rustcVer = & rustc --version
    Write-Host "   [OK] rustc: $rustcVer" -ForegroundColor Green
} else {
    Write-Host "   [FAIL] rustc was not found in PATH!" -ForegroundColor Red
}

$cargoCmd = Get-Command "cargo" -ErrorAction SilentlyContinue
if ($cargoCmd) {
    $cargoVer = & cargo --version
    Write-Host "   [OK] cargo: $cargoVer" -ForegroundColor Green
} else {
    Write-Host "   [FAIL] cargo was not found in PATH!" -ForegroundColor Red
}

# 3. Microsoft C++ Linker (link.exe)
Write-Host ""
Write-Host "[3] Microsoft Visual C++ Build Tools (MSVC Linker):" -ForegroundColor Yellow
$linkCmd = Get-Command "link.exe" -ErrorAction SilentlyContinue
if ($linkCmd) {
    Write-Host "   [OK] link.exe: $($linkCmd.Source)" -ForegroundColor Green
} else {
    $VsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $VsWhere) {
        $linkPaths = & $VsWhere -latest -find **/link.exe
        if ($linkPaths) {
            Write-Host "   [OK] link.exe found at: $($linkPaths[0])" -ForegroundColor Green
        } else {
            Write-Host "   [FAIL] MSVC C++ Build Tools not found." -ForegroundColor Red
            Write-Host "          Please install 'Desktop development with C++' via Visual Studio Installer." -ForegroundColor Gray
        }
    } else {
        Write-Host "   [FAIL] Visual Studio Installer / Build Tools not found." -ForegroundColor Red
    }
}

# 4. Microsoft Edge WebView2 Runtime
Write-Host ""
Write-Host "[4] Microsoft Edge WebView2 Runtime:" -ForegroundColor Yellow
$regPath = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-F501-47DD-9A06-A8E34CEE7906}"
if (Test-Path $regPath) {
    $wvVersion = (Get-ItemProperty -Path $regPath -Name "pv" -ErrorAction SilentlyContinue).pv
    Write-Host "   [OK] WebView2 Runtime is available (Version: $wvVersion)" -ForegroundColor Green
} else {
    Write-Host "   [INFO] Using default system WebView2 Runtime." -ForegroundColor Gray
}

Write-Host ""
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "   Diagnostics Completed                            " -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan
