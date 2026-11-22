#!/usr/bin/env bash
# DesktopBubbleChat - Linux Process Terminator
set -e

echo -e "\033[31m====================================================\033[0m"
echo -e "\033[31m   DesktopBubbleChat - Linux Process Terminator     \033[0m"
echo -e "\033[31m====================================================\033[0m"

STOPPED=0

# Terminate Tauri Host application and its direct WebKit child processes only
HOST_PIDS=$(pgrep -x "desktop-bubble-chat" 2>/dev/null || true)
if [ -n "$HOST_PIDS" ]; then
    for HPID in $HOST_PIDS; do
        pkill -9 -P "$HPID" 2>/dev/null || true
        kill -9 "$HPID" 2>/dev/null || true
    done
    echo -e "\033[32m[Stop] Terminated desktop-bubble-chat host and child WebKit processes.\033[0m"
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
