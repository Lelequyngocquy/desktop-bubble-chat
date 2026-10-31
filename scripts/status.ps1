# DesktopBubbleChat - System Resource & Process Monitor
[CmdletBinding()]
param (
    [switch]$Watch,
    [int]$IntervalSec = 2
)

function Get-AppProcessTree {
    $mainProcs = Get-Process -Name "DesktopBubbleChat", "desktop-bubble-chat" -ErrorAction SilentlyContinue

    if (-not $mainProcs) {
        return $null
    }

    $wvProcsCim = Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" -ErrorAction SilentlyContinue |
        Where-Object {
            $_.CommandLine -like "*com.desktopbubblechat.app*" -or
            $_.CommandLine -like "*DesktopBubbleChat*"
        }

    $processList = [System.Collections.Generic.List[PSCustomObject]]::new()

    foreach ($mp in $mainProcs) {
        $cpuVal = 0
        try { $cpuVal = [math]::Round($mp.CPU, 2) } catch {}

        $processList.Add([PSCustomObject]@{
            PID          = $mp.Id
            ProcessName  = $mp.ProcessName
            Category     = "Rust Host (Main App Core)"
            Type         = "Main Host"
            WorkingSetMB = [math]::Round($mp.WorkingSet64 / 1MB, 2)
            CPU_Sec      = $cpuVal
            Profile      = "Host"
        })
    }

    if ($wvProcsCim) {
        foreach ($cim in $wvProcsCim) {
            $proc = Get-Process -Id $cim.ProcessId -ErrorAction SilentlyContinue
            if (-not $proc) { continue }

            $cmd = $cim.CommandLine
            $type = "browser"
            $category = "WebView2 Broker"

            if ($cmd -match "--type=([a-zA-Z0-9_-]+)") {
                $type = $Matches[1]
            }

            if ($type -eq "gpu-process") {
                $category = "GPU Acceleration (Direct3D)"
            } elseif ($type -eq "utility") {
                if ($cmd -match "--utility-sub-type=([a-zA-Z0-9_.-]+)") {
                    $sub = $Matches[1]
                    if ($sub -like "*network*") { $category = "Utility: Network Service" }
                    elseif ($sub -like "*audio*") { $category = "Utility: Audio Service" }
                    elseif ($sub -like "*storage*") { $category = "Utility: Storage Service" }
                    else { $category = "Utility: $sub" }
                } else {
                    $category = "Utility Service"
                }
            } elseif ($type -eq "renderer") {
                $category = "Renderer (UI / Bubble / Chat)"
            } elseif ($type -eq "crashpad-handler") {
                $category = "Crashpad Handler"
            }

            $profileName = "Default"
            if ($cmd -match "messenger_(\d+)") {
                $profileName = "messenger_$($Matches[1])"
            } elseif ($cmd -match "profiles[/\\]([a-zA-Z0-9_-]+)") {
                $profileName = $Matches[1]
            }

            $cpuSec = 0
            try { $cpuSec = [math]::Round($proc.CPU, 2) } catch {}

            $processList.Add([PSCustomObject]@{
                PID          = $proc.Id
                ProcessName  = $proc.ProcessName
                Category     = $category
                Type         = $type
                WorkingSetMB = [math]::Round($proc.WorkingSet64 / 1MB, 2)
                CPU_Sec      = $cpuSec
                Profile      = $profileName
            })
        }
    }

    return $processList
}

function Show-StatusSnapshot {
    $processes = Get-AppProcessTree

    if (-not $processes) {
        Write-Host "====================================================" -ForegroundColor Red
        Write-Host "   DesktopBubbleChat: KHONG HOAT DONG               " -ForegroundColor Red
        Write-Host "====================================================" -ForegroundColor Red
        Write-Host "[INFO] Khong tim thay tien trinh DesktopBubbleChat nao dang chay." -ForegroundColor Yellow
        Write-Host "[HUONG DAN] Chay '.\scripts\start.ps1' de khoi dong ung dung." -ForegroundColor Gray
        return $false
    }

    $totalRAM_MB = [math]::Round(($processes | Measure-Object -Property WorkingSetMB -Sum).Sum, 2)
    $totalRAM_GB = [math]::Round($totalRAM_MB / 1024, 2)
    $totalCount = $processes.Count

    $rustCoreRAM = ($processes | Where-Object { $_.Category -like "*Rust Host*" } | Measure-Object -Property WorkingSetMB -Sum).Sum
    $renderersRAM = ($processes | Where-Object { $_.Category -like "*Renderer*" } | Measure-Object -Property WorkingSetMB -Sum).Sum
    $gpuRAM = ($processes | Where-Object { $_.Category -like "*GPU*" } | Measure-Object -Property WorkingSetMB -Sum).Sum
    $utilityRAM = ($processes | Where-Object { $_.Category -like "*Utility*" } | Measure-Object -Property WorkingSetMB -Sum).Sum
    $brokerRAM = ($processes | Where-Object { $_.Category -like "*Broker*" } | Measure-Object -Property WorkingSetMB -Sum).Sum

    $nowStr = Get-Date -Format "HH:mm:ss dd/MM/yyyy"

    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host "   DESKTOPBUBBLECHAT - BAO CAO TAI NGUYEN HE THONG [$nowStr]   " -ForegroundColor Cyan
    Write-Host "================================================================================" -ForegroundColor Cyan

    Write-Host ""
    Write-Host " [TONG QUAN TAI NGUYEN]" -ForegroundColor Yellow
    Write-Host "   * Tong RAM chiem dung : " -NoNewline
    Write-Host "$totalRAM_MB MB (~$totalRAM_GB GB)" -ForegroundColor Green
    Write-Host "   * Tong so tien trinh  : " -NoNewline
    Write-Host "$totalCount tien trinh" -ForegroundColor White
    Write-Host ""

    Write-Host " [PHAN BO BO NHO THEO VAI TRO]" -ForegroundColor Yellow
    Write-Host ("   - Rust Host (App Core)     : " + [string]::Format("{0:N2} MB", $rustCoreRAM)) -ForegroundColor Cyan
    Write-Host ("   - Web Renderers (Giao dien): " + [string]::Format("{0:N2} MB", $renderersRAM)) -ForegroundColor Magenta
    Write-Host ("   - GPU Engine (Direct3D)    : " + [string]::Format("{0:N2} MB", $gpuRAM)) -ForegroundColor DarkCyan
    Write-Host ("   - WebView2 Broker Processes: " + [string]::Format("{0:N2} MB", $brokerRAM)) -ForegroundColor Gray
    Write-Host ("   - Utility Services (Net/Aud): " + [string]::Format("{0:N2} MB", $utilityRAM)) -ForegroundColor DarkYellow
    Write-Host ""

    Write-Host " [DANH SACH CHI TIET TUNG TIEN TRINH]" -ForegroundColor Yellow

    $sorted = $processes | Sort-Object WorkingSetMB -Descending

    $formatTable = $sorted | Select-Object `
        @{Name="PID"; Expression={$_.PID}}, `
        @{Name="Vai tro / Phan loai"; Expression={$_.Category}}, `
        @{Name="Profile"; Expression={$_.Profile}}, `
        @{Name="RAM (MB)"; Expression={[string]::Format("{0:N2}", $_.WorkingSetMB)}}, `
        @{Name="CPU (s)"; Expression={[string]::Format("{0:N2}", $_.CPU_Sec)}}

    $formatTable | Format-Table -AutoSize | Out-String | Write-Host -ForegroundColor White

    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor Gray
    Write-Host "[GHI CHU] Khi dong goi ban Release (DesktopBubbleChat.exe), toan bo tien trinh" -ForegroundColor Gray
    Write-Host "          tren se duoc Task Manager Windows tu dong gom vao 1 dong kem Logo." -ForegroundColor Gray
    Write-Host ""

    return $true
}

if ($Watch) {
    Write-Host "[WATCH MODE] Dang theo doi lien tuc moi $IntervalSec giay (Nhan Ctrl+C de dung)..." -ForegroundColor Yellow
    Start-Sleep -Milliseconds 600
    try {
        while ($true) {
            Clear-Host
            $isRunning = Show-StatusSnapshot
            if (-not $isRunning) {
                break
            }
            Write-Host "[INFO] Dang theo doi... Cap nhat moi $IntervalSec giay. Nhan Ctrl+C de thoat." -ForegroundColor DarkGray
            Start-Sleep -Seconds $IntervalSec
        }
    } catch {
    }
} else {
    Show-StatusSnapshot | Out-Null
}
