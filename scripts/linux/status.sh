#!/usr/bin/env bash
# DesktopBubbleChat - Linux System Resource & Process Monitor
set -e

echo -e "\033[36m====================================================\033[0m"
echo -e "\033[36m   DesktopBubbleChat - Linux Process & RAM Monitor  \033[0m"
echo -e "\033[36m====================================================\033[0m"

HOST_PID=$(pgrep -f "desktop-bubble-chat" | head -n 1 || true)
if [ -z "$HOST_PID" ]; then
    echo -e "\033[31m[Status] DesktopBubbleChat: NOT RUNNING\033[0m"
    echo -e "\033[90m[Hint] Run 'bash scripts/linux/start.sh' to launch.\033[0m\n"
    exit 0
fi

printf "%-8s %-25s %-22s %-12s %-10s\n" "PID" "PROCESS" "CATEGORY" "PROFILE" "RAM (MB)"
printf "%-8s %-25s %-22s %-12s %-10s\n" "--------" "-------------------------" "----------------------" "------------" "----------"

TOTAL_RAM_KB=0
PROCS=$(ps -eo pid,rss,comm --no-headers | grep -E "desktop-bubble|WebKitWeb|WebKitNetwork|WebKitGPU" || true)

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
        *desktop-bubble*)
            CATEGORY="Rust Host Core"
            PROFILE="Host"
            ;;
        *WebKitWeb*)
            CATEGORY="WebKit Webview"
            PROFILE="Dashboard"
            ;;
        *WebKitNetwork*)
            CATEGORY="Network Service"
            PROFILE="Shared"
            ;;
        *WebKitGPU*)
            CATEGORY="GPU Acceleration"
            PROFILE="Shared"
            ;;
    esac

    # Inspect /proc/<pid>/maps for account profile index
    if [ -d "/proc/$PID" ]; then
        for idx in 01 02 03 04 05; do
            if grep -q "messenger_$idx" "/proc/$PID/maps" 2>/dev/null; then
                PROFILE="Account $idx"
                break
            fi
        done
    fi

    printf "%-8s %-25s %-22s %-12s %-10s\n" "$PID" "$COMM" "$CATEGORY" "$PROFILE" "$RAM_MB"
done <<< "$PROCS"

TOTAL_MB=$(awk "BEGIN {printf \"%.2f\", $TOTAL_RAM_KB/1024}")
echo -e "\033[36m----------------------------------------------------\033[0m"
echo -e "\033[32m[Summary] Total RAM Allocated: $TOTAL_MB MB\033[0m\n"
