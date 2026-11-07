import { spawnSync } from "node:child_process";
import os from "node:os";

// Collects running DesktopBubbleChat processes on Windows
function getWindowsProcesses() {
  const list = [];
  try {
    const mainPs = spawnSync("powershell", ["-NoProfile", "-Command", "Get-Process -Name 'DesktopBubbleChat','desktop-bubble-chat' -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,WorkingSet64,CPU | ConvertTo-Json"], { encoding: "utf8" });
    if (mainPs.stdout && mainPs.stdout.trim()) {
      const parsed = JSON.parse(mainPs.stdout.trim());
      const items = Array.isArray(parsed) ? parsed : [parsed];
      for (const item of items) {
        list.push({
          pid: item.Id,
          name: item.ProcessName,
          category: "Rust Host (Main App Core)",
          profile: "Host",
          ramMb: Math.round(((item.WorkingSet64 || 0) / 1024 / 1024) * 100) / 100,
          cpuSec: Math.round((item.CPU || 0) * 100) / 100,
        });
      }
    }
    const wvPs = spawnSync("powershell", ["-NoProfile", "-Command", "Get-CimInstance Win32_Process -Filter \"Name = 'msedgewebview2.exe'\" -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -like '*com.desktopbubblechat.app*' -or $_.CommandLine -like '*DesktopBubbleChat*' } | Select-Object ProcessId,Name,CommandLine,WorkingSetSize | ConvertTo-Json"], { encoding: "utf8" });
    if (wvPs.stdout && wvPs.stdout.trim()) {
      const parsed = JSON.parse(wvPs.stdout.trim());
      const items = Array.isArray(parsed) ? parsed : [parsed];
      for (const item of items) {
        const cmd = item.CommandLine || "";
        let category = "WebView2 Broker";
        if (cmd.includes("--type=gpu-process")) category = "GPU Acceleration";
        else if (cmd.includes("--type=renderer")) category = "Renderer (UI/Chat)";
        else if (cmd.includes("--type=utility")) category = "Utility Service";
        else if (cmd.includes("--type=crashpad-handler")) category = "Crashpad Handler";
        let profile = "Default";
        const m = cmd.match(/messenger_(\d+)/);
        if (m) profile = `messenger_${m[1]}`;
        list.push({
          pid: item.ProcessId,
          name: item.Name,
          category,
          profile,
          ramMb: Math.round(((item.WorkingSetSize || 0) / 1024 / 1024) * 100) / 100,
          cpuSec: 0,
        });
      }
    }
  } catch {}
  return list;
}

// Collects running DesktopBubbleChat processes on macOS and Linux
function getUnixProcesses() {
  const list = [];
  try {
    const res = spawnSync("ps", ["-axo", "pid,rss,command"], { encoding: "utf8" });
    if (res.stdout) {
      for (const line of res.stdout.split("\n")) {
        const trimmed = line.trim();
        if (!trimmed) continue;
        const parts = trimmed.split(/\s+/);
        const pid = parseInt(parts[0], 10);
        const rssKb = parseInt(parts[1], 10) || 0;
        const cmd = parts.slice(2).join(" ");
        if (cmd.includes("DesktopBubbleChat") || cmd.includes("desktop-bubble-chat") || cmd.includes("WebKit") || cmd.includes("webkit")) {
          let category = "Process";
          let profile = "Default";
          if (cmd.includes("desktop-bubble-chat") || cmd.includes("DesktopBubbleChat.app")) category = "Rust Host (Main App Core)";
          else if (cmd.includes("WebContent") || cmd.includes("WebKitWebProcess")) category = "Renderer (UI/Chat)";
          else if (cmd.includes("Networking") || cmd.includes("WebKitNetworkProcess")) category = "Utility: Network";
          const m = cmd.match(/messenger_(\d+)/);
          if (m) profile = `messenger_${m[1]}`;
          list.push({
            pid,
            name: parts[2] || "app",
            category,
            profile,
            ramMb: Math.round((rssKb / 1024) * 100) / 100,
            cpuSec: 0,
          });
        }
      }
    }
  } catch {}
  return list;
}

// Formats and displays a snapshot of process resources and memory breakdown
export function showStatusSnapshot() {
  const isWin = os.platform() === "win32";
  const procs = isWin ? getWindowsProcesses() : getUnixProcesses();
  if (!procs.length) {
    console.log("====================================================");
    console.log("   \x1b[31mDesktopBubbleChat: NOT RUNNING\x1b[0m");
    console.log("====================================================");
    console.log("\x1b[33m[Info] No active DesktopBubbleChat processes found.\x1b[0m");
    console.log("\x1b[90m[Hint] Run 'npm run app:start' to launch application.\x1b[0m\n");
    return false;
  }
  procs.sort((a, b) => b.ramMb - a.ramMb);
  const totalRam = Math.round(procs.reduce((acc, p) => acc + p.ramMb, 0) * 100) / 100;
  const now = new Date().toLocaleTimeString();
  console.log("====================================================");
  console.log(`   SYSTEM RESOURCE & PROCESS MONITOR [${now}]`);
  console.log("====================================================");
  console.log("\x1b[33m[PROCESS DETAILS]\x1b[0m");
  console.log("PID      | RAM (MB) | Profile        | Role");
  console.log("---------+----------+----------------+--------------------------");
  for (const p of procs) {
    const pidStr = String(p.pid).padEnd(8);
    const ramStr = `${p.ramMb.toFixed(2)} MB`.padEnd(9);
    const profStr = p.profile.padEnd(14);
    console.log(`${pidStr} | ${ramStr} | ${profStr} | ${p.category}`);
  }
  console.log("\n\x1b[33m[TOTAL USAGE]\x1b[0m");
  console.log(`   * Total RAM Consumption : \x1b[32m${totalRam.toFixed(2)} MB\x1b[0m (~${(totalRam / 1024).toFixed(2)} GB)`);
  console.log(`   * Active Processes Count: ${procs.length} process(es)\n`);
  return true;
}

// Entrypoint supporting one-shot status or watch mode
if (process.argv[1]?.endsWith("status.mjs")) {
  const isWatch = process.argv.includes("--watch") || process.argv.includes("-w");
  if (isWatch) {
    console.log("[Watch] Monitoring every 2 seconds. Press Ctrl+C to stop.\n");
    setInterval(() => {
      console.clear();
      showStatusSnapshot();
    }, 2000);
  } else {
    showStatusSnapshot();
  }
}
