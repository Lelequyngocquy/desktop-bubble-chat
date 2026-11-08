#!/usr/bin/env bash
# DesktopBubbleChat - Linux Process Terminator
set -e

echo -e "\033[31m====================================================\033[0m"
echo -e "\033[31m   DesktopBubbleChat - Linux Process Terminator     \033[0m"
echo -e "\033[31m====================================================\033[0m"

STOPPED=0

# Terminate Tauri Host application
if pkill -9 -f "desktop-bubble-chat" 2>/dev/null; then
    echo -e "\033[32m[Stop] Terminated desktop-bubble-chat host process.\033[0m"
    STOPPED=$((STOPPED + 1))
fi

# Terminate orphaned WebKit child processes
if pkill -9 -f "WebKitWebProcess" 2>/dev/null; then
    echo -e "\033[32m[Stop] Terminated WebKitWebProcess renderers.\033[0m"
    STOPPED=$((STOPPED + 1))
fi
if pkill -9 -f "WebKitNetworkProcess" 2>/dev/null; then
    echo -e "\033[32m[Stop] Terminated WebKitNetworkProcess services.\033[0m"
    STOPPED=$((STOPPED + 1))
fi

# Free port 1420
if command -v fuser >/dev/null 2>&1; then
    if fuser -k 1420/tcp 2>/dev/null; then
        echo -e "\033[32m[Stop] Freed port 1420 via fuser.\033[0m"
        STOPPED=$((STOPPED + 1))
    fi
elif command -v lsof >/dev/null 2>&1; then
    LSOF_PIDS=$(lsof -ti:1420 2>/dev/null || true)
    if [ -n "$LSOF_PIDS" ]; then
        echo "$LSOF_PIDS" | xargs -r kill -9 2>/dev/null || true
        echo -e "\033[32m[Stop] Freed port 1420 via lsof.\033[0m"
        STOPPED=$((STOPPED + 1))
    fi
fi

echo -e "\033[36m[Stop] Completed. Total actions taken: $STOPPED\033[0m\n"
