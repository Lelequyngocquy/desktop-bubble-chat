import { spawn, execSync } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");

// Cleans up existing instances and occupied port 1420
function cleanStaleProcesses() {
  const isWin = os.platform() === "win32";
  const targetNames = isWin ? ["DesktopBubbleChat.exe", "desktop-bubble-chat.exe"] : ["DesktopBubbleChat", "desktop-bubble-chat"];
  for (const name of targetNames) {
    try {
      if (isWin) {
        execSync(`taskkill /F /IM ${name} /T`, { stdio: "ignore" });
      } else {
        execSync(`pkill -9 -f ${name}`, { stdio: "ignore" });
      }
    } catch {}
  }
  if (isWin) {
    try {
      const netstat = execSync("netstat -ano -p tcp", { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
      for (const line of netstat.split("\n")) {
        if (line.includes(":1420 ") && line.includes("LISTENING")) {
          const parts = line.trim().split(/\s+/);
          const pid = parts[parts.length - 1];
          if (pid && pid !== "0") {
            try { execSync(`taskkill /F /PID ${pid}`, { stdio: "ignore" }); } catch {}
          }
        }
      }
    } catch {}
  } else {
    try {
      const lsof = execSync("lsof -ti:1420", { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
      for (const pid of lsof.trim().split("\n")) {
        if (pid) {
          try { execSync(`kill -9 ${pid}`, { stdio: "ignore" }); } catch {}
        }
      }
    } catch {}
  }
}

// Ensures cargo bin directory is in PATH
function ensureCargoPath() {
  const home = os.homedir();
  const cargoBin = path.join(home, ".cargo", "bin");
  if (fs.existsSync(cargoBin) && !process.env.PATH.includes(cargoBin)) {
    process.env.PATH = `${cargoBin}${path.delimiter}${process.env.PATH}`;
  }
}

// Waits for local HTTP server to return status 200
async function waitForHttpReady(url, maxRetries = 40, delayMs = 250) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const ok = await new Promise((resolve) => {
        const req = http.get(url, (res) => {
          resolve(res.statusCode === 200);
        });
        req.on("error", () => resolve(false));
        req.setTimeout(500, () => { req.destroy(); resolve(false); });
      });
      if (ok) return true;
    } catch {}
    await new Promise((r) => setTimeout(r, delayMs));
  }
  return false;
}

// Launches application launcher in specified mode (dev, prod, web)
export async function startApp(mode = "dev") {
  console.log("====================================================");
  console.log("   DesktopBubbleChat - Application Launcher         ");
  console.log(`   Mode: ${mode} | OS: ${os.platform()} (${os.arch()})`);
  console.log("====================================================");
  ensureCargoPath();
  cleanStaleProcesses();
  if (mode === "web") {
    console.log("\x1b[32m[Start] Starting Vite Web Dev Server...\x1b[0m");
    const vite = spawn("npm", ["run", "dev", "--", "--force"], { cwd: projectRoot, stdio: "inherit", shell: true });
    vite.on("exit", (code) => process.exit(code || 0));
    return;
  }
  if (mode === "prod") {
    const isWin = os.platform() === "win32";
    const isMac = os.platform() === "darwin";
    const relExe = isWin
      ? path.join(projectRoot, "src-tauri", "target", "release", "DesktopBubbleChat.exe")
      : isMac
        ? path.join(projectRoot, "src-tauri", "target", "release", "bundle", "macos", "DesktopBubbleChat.app", "Contents", "MacOS", "DesktopBubbleChat")
        : path.join(projectRoot, "src-tauri", "target", "release", "desktop-bubble-chat");
    if (fs.existsSync(relExe)) {
      console.log(`\x1b[32m[Start] Launching Release executable: ${relExe}\x1b[0m`);
      const child = spawn(relExe, [], { detached: true, stdio: "ignore" });
      child.unref();
      process.exit(0);
    } else {
      console.error(`\x1b[31m[Error] Release executable not found at: ${relExe}\x1b[0m`);
      console.error("\x1b[33m[Info] Please run 'npm run app:build' first to produce production binary.\x1b[0m");
      process.exit(1);
    }
  }
  process.env.RUST_LOG = "info,desktop_bubble_chat=debug,desktop_bubble_chat_lib=debug";
  process.env.RUST_BACKTRACE = "1";
  const logDir = path.join(projectRoot, "logs");
  if (!fs.existsSync(logDir)) {
    fs.mkdirSync(logDir, { recursive: true });
  }
  const feLogFile = path.join(logDir, "frontend.log");
  const feErrFile = path.join(logDir, "frontend-error.log");
  const outStream = fs.openSync(feLogFile, "w");
  const errStream = fs.openSync(feErrFile, "w");
  console.log("\x1b[36m[Start] Starting background Vite dev server (logs in logs/frontend.log)...\x1b[0m");
  const viteChild = spawn("npm", ["run", "dev"], {
    cwd: projectRoot,
    stdio: ["ignore", outStream, errStream],
    shell: true,
  });
  const cleanup = () => {
    try { viteChild.kill("SIGTERM"); } catch {}
    cleanStaleProcesses();
  };
  process.on("SIGINT", () => { cleanup(); process.exit(0); });
  process.on("SIGTERM", () => { cleanup(); process.exit(0); });
  process.on("exit", cleanup);
  console.log("[Start] Waiting for Vite frontend server on http://localhost:1420...");
  const isReady = await waitForHttpReady("http://localhost:1420");
  if (!isReady) {
    console.error("\x1b[31m[Error] Timeout: Vite server did not respond on http://localhost:1420\x1b[0m");
    cleanup();
    process.exit(1);
  }
  console.log("\x1b[32m[Success] Vite frontend server is ready on http://localhost:1420 (HTTP 200)\x1b[0m");
  console.log("====================================================");
  console.log("   STREAMING BACKEND LOGS (Rust / Tauri / OS)       ");
  console.log("====================================================");
  console.log("\x1b[33m[Info] Terminal focused on BACKEND LOGS (BE).\x1b[0m");
  console.log("\x1b[90m[Info] Frontend logs saved to logs/frontend.log\x1b[0m");
  console.log("\x1b[90m[Info] Press Ctrl+C to terminate application.\x1b[0m\n");
  const beConfigPath = path.join(projectRoot, "scripts", "tauri.be-logs.json");
  const tauriChild = spawn("npx", ["tauri", "dev", "--no-dev-server-wait", "--config", beConfigPath], {
    cwd: projectRoot,
    stdio: "inherit",
    shell: true,
  });
  tauriChild.on("exit", (code) => {
    cleanup();
    process.exit(code || 0);
  });
}

if (process.argv[1]?.endsWith("start.mjs")) {
  const modeArg = process.argv[2] || "dev";
  startApp(modeArg);
}
