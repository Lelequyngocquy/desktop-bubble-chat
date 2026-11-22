#!/usr/bin/env bash
# DesktopBubbleChat - macOS System Resource & Process Monitor
set -e

echo -e "\033[36m====================================================\033[0m"
echo -e "\033[36m   DesktopBubbleChat - macOS Process & RAM Monitor  \033[0m"
echo -e "\033[36m====================================================\033[0m"

HOST_PID=$(pgrep -x "DesktopBubbleChat" | head -n 1 || pgrep -x "desktop-bubble-chat" | head -n 1 || true)
if [ -z "$HOST_PID" ]; then
    echo -e "\033[31m[Status] DesktopBubbleChat: NOT RUNNING\033[0m"
    echo -e "\033[90m[Hint] Run 'bash scripts/macos/start.sh' to launch.\033[0m\n"
    exit 0
fi

printf "%-8s %-25s %-22s %-12s %-10s\n" "PID" "PROCESS" "CATEGORY" "PROFILE" "RAM (MB)"
printf "%-8s %-25s %-22s %-12s %-10s\n" "--------" "-------------------------" "----------------------" "------------" "----------"

TOTAL_RAM_KB=0
CHILD_PIDS=$(pgrep -P "$HOST_PID" 2>/dev/null | tr '\n' ',' | sed 's/,$//' || true)
PID_LIST="$HOST_PID"
if [ -n "$CHILD_PIDS" ]; then
    PID_LIST="${HOST_PID},${CHILD_PIDS}"
fi
PROCS=$(ps -p "$PID_LIST" -o pid=,rss=,command= 2>/dev/null || true)

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
        *DesktopBubbleChat*|*desktop-bubble-chat*)
            CATEGORY="Rust Host Core"
            PROFILE="Host"
            ;;
        *WebContent*)
            CATEGORY="WebKit Webview"
            PROFILE="WebView"
            ;;
        *Networking*)
            CATEGORY="Network Service"
            PROFILE="Shared"
            ;;
        *GPU*)
            CATEGORY="GPU Service"
            PROFILE="Shared"
            ;;
    esac

    printf "%-8s %-25s %-22s %-12s %-10s\n" "$PID" "$(basename "$COMM")" "$CATEGORY" "$PROFILE" "$RAM_MB"
done <<< "$PROCS"

TOTAL_MB=$(awk "BEGIN {printf \"%.2f\", $TOTAL_RAM_KB/1024}")
echo -e "\033[36m----------------------------------------------------\033[0m"
echo -e "\033[32m[Summary] Total RAM Allocated: $TOTAL_MB MB\033[0m\n"
