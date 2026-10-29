<#
.SYNOPSIS
    Terminates all running DesktopBubbleChat processes and related instances.
.DESCRIPTION
    Scans and terminates running processes including DesktopBubbleChat,
    background chat head instances, and stale dev servers on port 1420.
.PARAMETER Force
    Forces process termination without waiting. Default is true.
#>
[CmdletBinding()]
param (
    [switch]$Force = $true
)

Write-Host "====================================================" -ForegroundColor Red
Write-Host "   DesktopBubbleChat - Process Terminator           " -ForegroundColor Red
Write-Host "====================================================" -ForegroundColor Red

$TargetNames = @(
    "DesktopBubbleChat",
    "desktop-bubble-chat"
)

$StoppedCount = 0

foreach ($name in $TargetNames) {
    $processes = Get-Process -Name $name -ErrorAction SilentlyContinue
    if ($processes) {
        foreach ($proc in $processes) {
            $pidVal = $proc.Id
            $procName = $proc.ProcessName
            Write-Host "[INFO] Stopping process $procName (PID: $pidVal)..." -ForegroundColor Yellow
            try {
                if ($Force) {
                    Stop-Process -Id $pidVal -Force
                } else {
                    $proc.CloseMainWindow() | Out-Null
                    Start-Sleep -Milliseconds 500
                    if (-not $proc.HasExited) {
                        Stop-Process -Id $pidVal -Force
                    }
                }
                Write-Host "[SUCCESS] Stopped $procName (PID: $pidVal)" -ForegroundColor Green
                $StoppedCount++
            } catch {
                Write-Host "[ERROR] Failed to stop PID $pidVal - $_" -ForegroundColor Red
            }
        }
    }
}

# Check port 1420 (Vite Dev Server)
try {
    $Port1420 = Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue
    if ($Port1420) {
        $Pids = $Port1420 | Select-Object -ExpandProperty OwningProcess -Unique
        foreach ($p in $Pids) {
            $nodeProc = Get-Process -Id $p -ErrorAction SilentlyContinue
            if ($nodeProc -and ($nodeProc.ProcessName -eq "node")) {
                Write-Host "[INFO] Found Vite Dev Server on port 1420 (PID: $p)..." -ForegroundColor Yellow
                Stop-Process -Id $p -Force
                Write-Host "[SUCCESS] Stopped Vite Dev Server (PID: $p)" -ForegroundColor Green
                $StoppedCount++
            }
        }
    }
} catch {
    # Ignore if port cannot be queried
}

Write-Host ""
if ($StoppedCount -eq 0) {
    Write-Host "[INFO] No DesktopBubbleChat processes are currently running." -ForegroundColor Cyan
} else {
    Write-Host "[SUCCESS] Successfully stopped $StoppedCount process(es)." -ForegroundColor Green
}
