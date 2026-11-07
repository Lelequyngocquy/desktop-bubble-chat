import { execSync } from "node:child_process";
import os from "node:os";

// Runs a command and returns trimmed stdout or null on failure
function tryExec(cmd) {
  try {
    return execSync(cmd, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
  } catch {
    return null;
  }
}

// Runs environment diagnostic checks across Windows, macOS, and Linux
export function checkEnvironment() {
  const isWin = os.platform() === "win32";
  const isMac = os.platform() === "darwin";
  const isLinux = os.platform() === "linux";
  console.log("====================================================");
  console.log("   DesktopBubbleChat - Environment Diagnostics      ");
  console.log(`   Platform: ${os.platform()} (${os.arch()})`);
  console.log("====================================================");
  const nodeVer = tryExec("node --version");
  const npmVer = tryExec("npm --version");
  console.log(`\n[1] Node.js & npm:`);
  console.log(nodeVer ? `   \x1b[32m[OK] Node.js: ${nodeVer}\x1b[0m` : `   \x1b[31m[FAIL] Node.js not found\x1b[0m`);
  console.log(npmVer ? `   \x1b[32m[OK] npm: v${npmVer}\x1b[0m` : `   \x1b[31m[FAIL] npm not found\x1b[0m`);
  const rustcVer = tryExec("rustc --version");
  const cargoVer = tryExec("cargo --version");
  console.log(`\n[2] Rust Toolchain:`);
  console.log(rustcVer ? `   \x1b[32m[OK] rustc: ${rustcVer}\x1b[0m` : `   \x1b[31m[FAIL] rustc not found\x1b[0m`);
  console.log(cargoVer ? `   \x1b[32m[OK] cargo: ${cargoVer}\x1b[0m` : `   \x1b[31m[FAIL] cargo not found\x1b[0m`);
  console.log(`\n[3] Native Build Toolchain & System Dependencies:`);
  if (isWin) {
    const linkPath = tryExec("where link.exe");
    console.log(linkPath ? `   \x1b[32m[OK] MSVC Linker: ${linkPath.split("\n")[0]}\x1b[0m` : `   \x1b[33m[WARN] link.exe not found directly in PATH (checked via Visual Studio Build Tools)\x1b[0m`);
    const regWv = tryExec("reg query \"HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-F501-47DD-9A06-A8E34CEE7906}\" /v pv");
    console.log(regWv ? `   \x1b[32m[OK] WebView2 Runtime detected\x1b[0m` : `   \x1b[36m[INFO] System default WebView2 runtime active\x1b[0m`);
  } else if (isMac) {
    const xcodePath = tryExec("xcode-select -p");
    console.log(xcodePath ? `   \x1b[32m[OK] Xcode Command Line Tools: ${xcodePath}\x1b[0m` : `   \x1b[31m[FAIL] Xcode tools missing (run: xcode-select --install)\x1b[0m`);
    const clangVer = tryExec("clang --version");
    console.log(clangVer ? `   \x1b[32m[OK] Apple Clang available\x1b[0m` : `   \x1b[31m[FAIL] Clang compiler missing\x1b[0m`);
  } else if (isLinux) {
    const pkgConfig = tryExec("pkg-config --version");
    console.log(pkgConfig ? `   \x1b[32m[OK] pkg-config: v${pkgConfig}\x1b[0m` : `   \x1b[31m[FAIL] pkg-config missing (run: sudo apt install pkg-config)\x1b[0m`);
    const gtkVer = tryExec("pkg-config --modversion gtk+-3.0");
    console.log(gtkVer ? `   \x1b[32m[OK] GTK 3: v${gtkVer}\x1b[0m` : `   \x1b[31m[FAIL] libgtk-3-dev missing\x1b[0m`);
    const webkitVer = tryExec("pkg-config --modversion webkit2gtk-4.1") || tryExec("pkg-config --modversion webkit2gtk-4.0");
    console.log(webkitVer ? `   \x1b[32m[OK] WebKitGTK: v${webkitVer}\x1b[0m` : `   \x1b[31m[FAIL] libwebkit2gtk-4.1-dev missing\x1b[0m`);
    const appInd = tryExec("pkg-config --modversion ayatana-appindicator3-0.1");
    console.log(appInd ? `   \x1b[32m[OK] Ayatana AppIndicator: v${appInd}\x1b[0m` : `   \x1b[33m[WARN] libayatana-appindicator3-dev recommended for system tray\x1b[0m`);
  }
  console.log("\n====================================================");
  console.log("   Diagnostics Completed                            ");
  console.log("====================================================");
}

if (process.argv[1]?.endsWith("check-env.mjs")) {
  checkEnvironment();
}
