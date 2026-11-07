import { execSync } from "node:child_process";

// Executes production build pipeline for frontend and Tauri backend
export function runProductionBuild() {
  console.log("=============================================");
  console.log("DesktopBubbleChat - Production Build Pipeline");
  console.log("=============================================");
  console.log("\n[1/3] Building Frontend (TypeScript & Vite)...");
  execSync("npm run build", { stdio: "inherit" });
  console.log("\n[2/3] Verifying Rust Backend Codebase...");
  execSync("cargo check --manifest-path src-tauri/Cargo.toml", { stdio: "inherit" });
  console.log("\n[3/3] Packaging Production Release with Tauri...");
  execSync("npx tauri build", { stdio: "inherit" });
  console.log("\n=============================================");
  console.log("PRODUCTION BUILD COMPLETED SUCCESSFULLY!");
  console.log("=============================================");
}

if (process.argv[1]?.endsWith("build.mjs")) {
  runProductionBuild();
}
