import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TitleBar } from "./components/layout/TitleBar";
import { Sidebar } from "./components/layout/Sidebar";
import { HomeWorkspace } from "./components/home/HomeWorkspace";
import { BubbleWindow } from "./components/bubble/BubbleWindow";
import { useUnreadListener } from "./hooks/useUnreadListener";

export const App: React.FC = () => {
  const [windowLabel, setWindowLabel] = useState<string>("main");

  // Lắng nghe sự kiện đồng bộ badge trên tất cả các cửa sổ
  useUnreadListener();

  useEffect(() => {
    try {
      const win = getCurrentWindow();
      if (win && win.label) {
        setWindowLabel(win.label);
      }
    } catch {
      // In standard browser Vite preview mode, default to 'main'
    }
  }, []);

  // Check if this window is the floating bubble
  const isBubbleWindow =
    windowLabel === "bubble" ||
    window.location.search.includes("window=bubble") ||
    window.location.hash === "#bubble";

  if (isBubbleWindow) {
    return (
      <div className="w-screen h-screen flex items-center justify-center bg-transparent overflow-hidden select-none">
        <BubbleWindow />
      </div>
    );
  }

  // Default: Desktop Main App (Home Page from Figma Node #3:1145)
  return (
    <div className="flex flex-col h-screen w-screen bg-[#EEF2F9] overflow-hidden text-[#202733] select-none font-sans border border-[#E2E7EF] shadow-window">
      {/* 1. Custom Title Bar (Figma Node #3:1146) */}
      <TitleBar />

      {/* 2. Main Workspace (Sidebar + Home Page Content) */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar Navigation (Figma Node #3:1159) */}
        <Sidebar />

        {/* Home Workspace (Figma Node #3:1197) */}
        <HomeWorkspace />
      </div>
    </div>
  );
};

export default App;
