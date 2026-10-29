import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TitleBar } from "./components/layout/TitleBar";
import { Sidebar } from "./components/layout/Sidebar";
import { HomeWorkspace } from "./components/home/HomeWorkspace";
import { ConnectionsWorkspace } from "./components/connections/ConnectionsWorkspace";
import { BubbleWindow } from "./components/bubble/BubbleWindow";
import { useUnreadListener } from "./hooks/useUnreadListener";
import { useAppStore } from "./stores/appStore";

export const App: React.FC = () => {
  const [windowLabel, setWindowLabel] = useState<string>("main");
  const { activeTab, fetchSystemUser } = useAppStore();

  // Lắng nghe sự kiện đồng bộ badge trên tất cả các cửa sổ
  useUnreadListener();

  useEffect(() => {
    // Lấy thông tin tài khoản Windows khi khởi chạy
    fetchSystemUser();

    try {
      const win = getCurrentWindow();
      if (win && win.label) {
        setWindowLabel(win.label);
      }
    } catch {
      // In standard browser Vite preview mode, default to 'main'
    }
  }, [fetchSystemUser]);

  // Check if this window is the floating bubble
  const isBubbleWindow =
    windowLabel === "bubble" ||
    window.location.search.includes("window=bubble") ||
    window.location.hash === "#bubble";

  if (isBubbleWindow) {
    return (
      <div className="w-16 h-16 flex items-center justify-center bg-transparent overflow-hidden select-none">
        <BubbleWindow />
      </div>
    );
  }

  // Default: Desktop Main App (Home Page / Connections / etc.)
  return (
    <div className="flex flex-col h-screen w-screen bg-[#EEF2F9] overflow-hidden text-[#202733] select-none font-sans border border-[#E2E7EF] shadow-window">
      {/* 1. Custom Title Bar (Figma Node #3:1146 & #20:1052) */}
      <TitleBar />

      {/* 2. Main Workspace (Sidebar + Dynamic Workspace Content) */}
      <div className="flex flex-1 min-h-0 min-w-0 overflow-hidden">
        {/* Sidebar Navigation (Figma Node #3:1159 & #20:1069) */}
        <Sidebar />

        {/* Dynamic Workspace */}
        {activeTab === "connections" ? (
          <ConnectionsWorkspace />
        ) : (
          <HomeWorkspace />
        )}
      </div>
    </div>
  );
};

export default App;
