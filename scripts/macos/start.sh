#!/usr/bin/env bash
# DesktopBubbleChat - macOS Application Launcher
set -e

MODE="${1:-dev}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

echo -e "\033[36m====================================================\033[0m"
echo -e "\033[36m   DesktopBubbleChat - macOS Application Launcher   \033[0m"
echo -e "\033[36m   Mode: $MODE | Root: $PROJECT_ROOT               \033[0m"
echo -e "\033[36m====================================================\033[0m"

# Ensure cargo is in PATH
if [ -d "$HOME/.cargo/bin" ] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Clean up stale processes & free port 1420
echo -e "\033[90m[Info] Cleaning stale processes & port 1420...\033[0m"
pkill -9 -f "DesktopBubbleChat" 2>/dev/null || true
if command -v lsof >/dev/null 2>&1; then
    lsof -ti:1420 | xargs -r kill -9 2>/dev/null || true
fi

if [ "$MODE" = "web" ]; then
    echo -e "\033[32m[Start] Starting Vite Web Dev Server...\033[0m"
    exec npm run dev -- --force
fi

if [ "$MODE" = "prod" ]; then
    PROD_APP="$PROJECT_ROOT/src-tauri/target/release/bundle/macos/DesktopBubbleChat.app"
    if [ -d "$PROD_APP" ]; then
        echo -e "\033[32m[Start] Launching Release App Bundle: $PROD_APP\033[0m"
        open "$PROD_APP"
        exit 0
    else
        echo -e "\033[31m[Error] Release bundle not found at: $PROD_APP\033[0m"
        echo -e "\033[33m[Info] Please run 'bash scripts/macos/build.sh' first.\033[0m"
        exit 1
    fi
fi

# Development mode
export RUST_LOG="info,desktop_bubble_chat=debug,desktop_bubble_chat_lib=debug"
export RUST_BACKTRACE="1"

mkdir -p "$PROJECT_ROOT/logs"
FE_LOG="$PROJECT_ROOT/logs/frontend.log"
FE_ERR="$PROJECT_ROOT/logs/frontend-error.log"

echo -e "\033[36m[Start] Starting background Vite dev server (logs in logs/frontend.log)...\033[0m"
npm run dev > "$FE_LOG" 2> "$FE_ERR" &
VITE_PID=$!

cleanup() {
    echo -e "\n\033[33m[Stop] Terminating application & Vite server (PID: $VITE_PID)...\033[0m"
    kill "$VITE_PID" 2>/dev/null || true
    pkill -9 -f "DesktopBubbleChat" 2>/dev/null || true
    if command -v lsof >/dev/null 2>&1; then
        lsof -ti:1420 2>/dev/null | xargs -r kill -9 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

echo "[Start] Waiting for Vite frontend server on http://localhost:1420..."
READY=0
for i in {1..40}; do
    if command -v curl >/dev/null 2>&1; then
        STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:1420 || true)
        if [ "$STATUS" = "200" ]; then
            READY=1
            break
        fi
    fi
    sleep 0.25
done

if [ "$READY" -ne 1 ]; then
    echo -e "\033[31m[Error] Timeout: Vite server did not respond on http://localhost:1420\033[0m"
    exit 1
fi

echo -e "\033[32m[Success] Vite frontend server is ready on http://localhost:1420 (HTTP 200)\033[0m"
echo -e "\033[36m====================================================\033[0m"
echo -e "\033[36m   STREAMING BACKEND LOGS (Rust / Tauri / Cocoa)    \033[0m"
echo -e "\033[36m====================================================\033[0m"
echo -e "\033[33m[Info] Terminal focused on BACKEND LOGS (BE).\033[0m"
echo -e "\033[90m[Info] Frontend logs saved to logs/frontend.log\033[0m"
echo -e "\033[90m[Info] Press Ctrl+C to terminate application.\033[0m\n"

npx tauri dev --no-dev-server-wait --config "$PROJECT_ROOT/scripts/tauri.be-logs.json"
