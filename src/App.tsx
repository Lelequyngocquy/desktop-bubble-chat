import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TitleBar } from "./components/layout/TitleBar";
import { Sidebar } from "./components/layout/Sidebar";
import { HomeWorkspace } from "./components/home";
import { ConnectionsWorkspace } from "./components/connections";
import { ChatHeadsWorkspace } from "./components/chatheads";
import { NotificationsWorkspace } from "./components/notifications";
import { SettingsWorkspace } from "./components/settings";
import { HelpWorkspace } from "./components/help";
import { BubbleWindow } from "./components/bubble/BubbleWindow";
import { BubbleSpawnIndicator } from "./components/common/BubbleSpawnIndicator";
import { useBubbleAutoRestore } from "./hooks/bubble";
import { useUnreadListener } from "./hooks/useUnreadListener";
import { useSystemUser } from "./hooks/useSystemUser";
import { useNavigationStore } from "./stores/navigationStore";

const isSearchBubble =
  typeof window !== "undefined" &&
  (window.location.search.includes("window=bubble") ||
    window.location.hash.includes("bubble"));

const DashboardApp: React.FC = () => {
  const activeTab = useNavigationStore((s) => s.activeTab);

  // Initialize background system listeners & delayed bubble auto-restore (Main Dashboard only)
  useUnreadListener();
  useSystemUser();
  useBubbleAutoRestore();

  return (
    <div className="flex flex-col h-screen w-screen bg-[#EEF2F9] overflow-hidden text-[#202733] select-none font-sans border border-[#E2E7EF] shadow-window">
      {/* 1. Custom Title Bar */}
      <TitleBar />

      {/* 2. Main Workspace (Sidebar + Dynamic Workspace Content) */}
      <div className="flex flex-1 min-h-0 min-w-0 overflow-hidden">
        <Sidebar />
        {activeTab === "connections" ? (
          <ConnectionsWorkspace />
        ) : activeTab === "chat-heads" ? (
          <ChatHeadsWorkspace />
        ) : activeTab === "notifications" ? (
          <NotificationsWorkspace />
        ) : activeTab === "settings" ? (
          <SettingsWorkspace />
        ) : activeTab === "help" ? (
          <HelpWorkspace />
        ) : (
          <HomeWorkspace />
        )}
      </div>

      {/* 3. Dynamic Bubble Spawn Directional Arrow Indicator */}
      <BubbleSpawnIndicator />
    </div>
  );
};

export const App: React.FC = () => {
  const [windowLabel, setWindowLabel] = useState<string>(() => {
    try {
      const win = getCurrentWindow();
      if (win && win.label) return win.label;
    } catch {}
    return "main";
  });

  useEffect(() => {
    try {
      const win = getCurrentWindow();
      if (win && win.label) {
        setWindowLabel(win.label);
      }
    } catch {}
  }, []);

  const isBubbleWindow =
    isSearchBubble ||
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

  return <DashboardApp />;
};

export default App;
