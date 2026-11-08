#!/usr/bin/env bash
# DesktopBubbleChat - Linux Environment Diagnostics
set -e

echo -e "\033[36m====================================================\033[0m"
echo -e "\033[36m   DesktopBubbleChat - Linux Environment Check      \033[0m"
echo -e "\033[36m====================================================\033[0m"

# Ensure cargo is in PATH
if [ -d "$HOME/.cargo/bin" ] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo -e "\n\033[33m[1] Node.js & npm:\033[0m"
if command -v node >/dev/null 2>&1; then
    echo -e "   \033[32m[OK] Node.js: $(node --version)\033[0m"
else
    echo -e "   \033[31m[FAIL] Node.js is not installed!\033[0m"
fi
if command -v npm >/dev/null 2>&1; then
    echo -e "   \033[32m[OK] npm: v$(npm --version)\033[0m"
else
    echo -e "   \033[31m[FAIL] npm is not installed!\033[0m"
fi

echo -e "\n\033[33m[2] Rust Toolchain:\033[0m"
if command -v rustc >/dev/null 2>&1; then
    echo -e "   \033[32m[OK] rustc: $(rustc --version)\033[0m"
else
    echo -e "   \033[31m[FAIL] rustc is not installed!\033[0m"
fi
if command -v cargo >/dev/null 2>&1; then
    echo -e "   \033[32m[OK] cargo: $(cargo --version)\033[0m"
else
    echo -e "   \033[31m[FAIL] cargo is not installed!\033[0m"
fi

echo -e "\n\033[33m[3] Linux Native Dependencies:\033[0m"
for pkg in "gtk+-3.0:libgtk-3-dev" "webkit2gtk-4.1:libwebkit2gtk-4.1-dev" "ayatana-appindicator3-0.1:libayatana-appindicator3-dev"; do
    NAME="${pkg%%:*}"
    HINT="${pkg##*:}"
    if pkg-config --exists "$NAME" 2>/dev/null; then
        VER=$(pkg-config --modversion "$NAME")
        echo -e "   \033[32m[OK] $NAME: v$VER\033[0m"
    else
        echo -e "   \033[33m[WARN] $NAME missing (install: sudo apt install $HINT)\033[0m"
    fi
done

echo -e "\n\033[36m====================================================\033[0m"
echo -e "\033[36m   Diagnostics Completed                            \033[0m"
echo -e "\033[36m====================================================\033[0m\n"
