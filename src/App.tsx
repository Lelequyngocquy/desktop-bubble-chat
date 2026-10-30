import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TitleBar } from "./components/layout/TitleBar";
import { Sidebar } from "./components/layout/Sidebar";
import { HomeWorkspace } from "./components/home/HomeWorkspace";
import { ConnectionsWorkspace } from "./components/connections/ConnectionsWorkspace";
import { ChatHeadsWorkspace } from "./components/chatheads/ChatHeadsWorkspace";
import { NotificationsWorkspace } from "./components/notifications/NotificationsWorkspace";
import { SettingsWorkspace } from "./components/settings/SettingsWorkspace";
import { BubbleWindow } from "./components/bubble/BubbleWindow";
import { BubbleSpawnIndicator } from "./components/common/BubbleSpawnIndicator";
import { useBubbleAutoRestore } from "./hooks/bubble";
import { useUnreadListener } from "./hooks/useUnreadListener";
import { useSystemUser } from "./hooks/useSystemUser";
import { useNavigationStore } from "./stores/navigationStore";

export const App: React.FC = () => {
  const [windowLabel, setWindowLabel] = useState<string>("main");
  const activeTab = useNavigationStore((s) => s.activeTab);

  // Initialize background system listeners & delayed bubble auto-restore
  useUnreadListener();
  useSystemUser();
  useBubbleAutoRestore();

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

  // Check if this window instance is the floating bubble (supports bubble, bubble-01, bubble-02, etc.)
  const isBubbleWindow =
    windowLabel.startsWith("bubble") ||
    window.location.search.includes("window=bubble") ||
    window.location.hash.includes("bubble");

  if (isBubbleWindow) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-transparent overflow-hidden select-none">
        <BubbleWindow />
      </div>
    );
  }

  // Default: Desktop Main App (Home Page / Connections / etc.)
  return (
    <div className="flex flex-col h-screen w-screen bg-[#EEF2F9] overflow-hidden text-[#202733] select-none font-sans border border-[#E2E7EF] shadow-window">
      {/* 1. Custom Title Bar */}
      <TitleBar />

      {/* 2. Main Workspace (Sidebar + Dynamic Workspace Content) */}
      <div className="flex flex-1 min-h-0 min-w-0 overflow-hidden">
        {/* Sidebar Navigation */}
        <Sidebar />

        {/* Dynamic Workspace */}
        {activeTab === "connections" ? (
          <ConnectionsWorkspace />
        ) : activeTab === "chat-heads" ? (
          <ChatHeadsWorkspace />
        ) : activeTab === "notifications" ? (
          <NotificationsWorkspace />
        ) : activeTab === "settings" ? (
          <SettingsWorkspace />
        ) : (
          <HomeWorkspace />
        )}
      </div>

      {/* 3. Dynamic Bubble Spawn Directional Arrow Indicator */}
      <BubbleSpawnIndicator />
    </div>
  );
};

export default App;
