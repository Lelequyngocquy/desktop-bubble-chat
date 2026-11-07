import { execSync } from "node:child_process";
import os from "node:os";

// Stops all running DesktopBubbleChat processes and frees port 1420
export function stopAllProcesses() {
  const isWin = os.platform() === "win32";
  let stopped = 0;
  console.log("\x1b[31m[Stop] Stopping all DesktopBubbleChat processes...\x1b[0m");
  const targetNames = isWin ? ["DesktopBubbleChat.exe", "desktop-bubble-chat.exe"] : ["DesktopBubbleChat", "desktop-bubble-chat"];
  for (const name of targetNames) {
    try {
      if (isWin) {
        execSync(`taskkill /F /IM ${name} /T`, { stdio: "ignore" });
      } else {
        execSync(`pkill -9 -f ${name}`, { stdio: "ignore" });
      }
      stopped++;
      console.log(`\x1b[32m[Stop] Terminated process: ${name}\x1b[0m`);
    } catch {}
  }
  if (isWin) {
    try {
      const out = execSync("wmic process where \"name like '%msedgewebview2.exe%'\" get ProcessId,CommandLine", { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
      for (const line of out.split("\n")) {
        if (line.includes("com.desktopbubblechat.app") || line.includes("DesktopBubbleChat")) {
          const match = line.trim().match(/(\d+)$/);
          if (match) {
            try {
              execSync(`taskkill /F /PID ${match[1]}`, { stdio: "ignore" });
              stopped++;
            } catch {}
          }
        }
      }
    } catch {}
  }
  try {
    if (isWin) {
      const netstat = execSync("netstat -ano -p tcp", { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
      for (const line of netstat.split("\n")) {
        if (line.includes(":1420 ") && line.includes("LISTENING")) {
          const parts = line.trim().split(/\s+/);
          const pid = parts[parts.length - 1];
          if (pid && pid !== "0") {
            try {
              execSync(`taskkill /F /PID ${pid}`, { stdio: "ignore" });
              stopped++;
              console.log(`\x1b[32m[Stop] Freed port 1420 from PID: ${pid}\x1b[0m`);
            } catch {}
          }
        }
      }
    } else {
      try {
        const lsof = execSync("lsof -ti:1420", { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
        for (const pid of lsof.trim().split("\n")) {
          if (pid) {
            execSync(`kill -9 ${pid}`, { stdio: "ignore" });
            stopped++;
          }
        }
      } catch {}
    }
  } catch {}
  console.log(`\x1b[36m[Stop] Completed. Total actions taken: ${stopped}\x1b[0m`);
}

if (process.argv[1]?.endsWith("stop.mjs")) {
  stopAllProcesses();
}
