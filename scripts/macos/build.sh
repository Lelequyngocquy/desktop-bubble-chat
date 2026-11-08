#!/usr/bin/env bash
# DesktopBubbleChat - macOS Production Build Pipeline
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

echo -e "\033[36m=============================================\033[0m"
echo -e "\033[36mDesktopBubbleChat - macOS Production Build   \033[0m"
echo -e "\033[36m=============================================\033[0m"

# Ensure cargo is in PATH
if [ -d "$HOME/.cargo/bin" ] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo -e "\n\033[33m[1/3] Building Frontend (TypeScript & Vite)...\033[0m"
npm run build

echo -e "\n\033[33m[2/3] Verifying Rust Backend Codebase...\033[0m"
cargo check --manifest-path src-tauri/Cargo.toml

echo -e "\n\033[33m[3/3] Packaging Production Release with Tauri...\033[0m"
npx tauri build

echo -e "\n\033[32m=============================================\033[0m"
echo -e "\033[32mPRODUCTION BUILD COMPLETED SUCCESSFULLY!    \033[0m"
echo -e "\033[32m=============================================\033[0m\n"
