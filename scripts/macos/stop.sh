#!/usr/bin/env bash
# DesktopBubbleChat - macOS Process Terminator
set -e

echo -e "\033[31m====================================================\033[0m"
echo -e "\033[31m   DesktopBubbleChat - macOS Process Terminator     \033[0m"
echo -e "\033[31m====================================================\033[0m"

STOPPED=0

# Terminate Tauri Host application
if pkill -9 -f "DesktopBubbleChat" 2>/dev/null; then
    echo -e "\033[32m[Stop] Terminated DesktopBubbleChat host process.\033[0m"
    STOPPED=$((STOPPED + 1))
fi

# Free port 1420
if command -v lsof >/dev/null 2>&1; then
    LSOF_PIDS=$(lsof -ti:1420 2>/dev/null || true)
    if [ -n "$LSOF_PIDS" ]; then
        echo "$LSOF_PIDS" | xargs -r kill -9 2>/dev/null || true
        echo -e "\033[32m[Stop] Freed port 1420 via lsof.\033[0m"
        STOPPED=$((STOPPED + 1))
    fi
fi

echo -e "\033[36m[Stop] Completed. Total actions taken: $STOPPED\033[0m\n"
