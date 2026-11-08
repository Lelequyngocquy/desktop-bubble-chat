#!/usr/bin/env bash
# DesktopBubbleChat - macOS System Resource & Process Monitor
set -e

echo -e "\033[36m====================================================\033[0m"
echo -e "\033[36m   DesktopBubbleChat - macOS Process & RAM Monitor  \033[0m"
echo -e "\033[36m====================================================\033[0m"

HOST_PID=$(pgrep -f "DesktopBubbleChat" | head -n 1 || true)
if [ -z "$HOST_PID" ]; then
    echo -e "\033[31m[Status] DesktopBubbleChat: NOT RUNNING\033[0m"
    echo -e "\033[90m[Hint] Run 'bash scripts/macos/start.sh' to launch.\033[0m\n"
    exit 0
fi

printf "%-8s %-25s %-22s %-12s %-10s\n" "PID" "PROCESS" "CATEGORY" "PROFILE" "RAM (MB)"
printf "%-8s %-25s %-22s %-12s %-10s\n" "--------" "-------------------------" "----------------------" "------------" "----------"

TOTAL_RAM_KB=0
PROCS=$(ps -ax -o pid,rss,command | grep -E "DesktopBubbleChat|WebKit|com.apple.WebKit" | grep -v grep || true)

while IFS= read -r line; do
    [ -z "$line" ] && continue
    PID=$(echo "$line" | awk '{print $1}')
    RSS=$(echo "$line" | awk '{print $2}')
    COMM=$(echo "$line" | awk '{print $3}')
    RAM_MB=$(awk "BEGIN {printf \"%.2f\", $RSS/1024}")
    TOTAL_RAM_KB=$((TOTAL_RAM_KB + RSS))

    CATEGORY="Process"
    PROFILE="Host"
    case "$COMM" in
        *DesktopBubbleChat*)
            CATEGORY="Rust Host Core"
            PROFILE="Host"
            ;;
        *WebContent*)
            CATEGORY="WebKit Webview"
            PROFILE="Dashboard"
            ;;
        *Networking*)
            CATEGORY="Network Service"
            PROFILE="Shared"
            ;;
    esac

    printf "%-8s %-25s %-22s %-12s %-10s\n" "$PID" "$(basename "$COMM")" "$CATEGORY" "$PROFILE" "$RAM_MB"
done <<< "$PROCS"

TOTAL_MB=$(awk "BEGIN {printf \"%.2f\", $TOTAL_RAM_KB/1024}")
echo -e "\033[36m----------------------------------------------------\033[0m"
echo -e "\033[32m[Summary] Total RAM Allocated: $TOTAL_MB MB\033[0m\n"
