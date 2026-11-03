import React from "react";
import { TitleBar } from "./components/layout/TitleBar";
import { Sidebar } from "./components/layout/Sidebar";
import { HomeWorkspace } from "./components/home";
import { ConnectionsWorkspace } from "./components/connections";
import { ChatHeadsWorkspace } from "./components/chatheads";
import { NotificationsWorkspace } from "./components/notifications";
import { SettingsWorkspace } from "./components/settings";
import { HelpWorkspace } from "./components/help";
import { BubbleSpawnIndicator } from "./components/common/BubbleSpawnIndicator";
import { DashboardLoadingOverlay } from "./components/common/DashboardLoadingOverlay";
import { useBubbleAutoRestore, useDashboardBootstrap, useAutoUpdater } from "./hooks";
import { useUnreadListener } from "./hooks/useUnreadListener";
import { useSystemUser } from "./hooks/useSystemUser";
import { useNavigationStore } from "./stores/navigationStore";

export const App: React.FC = () => {
  const activeTab = useNavigationStore((s) => s.activeTab);

  // Khởi tạo các listener giám sát hệ thống, bootstrap vòng đời và khôi phục bong bóng nền (Dashboard chính)
  useDashboardBootstrap();
  useUnreadListener();
  useSystemUser();
  useBubbleAutoRestore();
  useAutoUpdater();

  return (
    <div className="flex flex-col h-screen w-screen bg-[#EEF2F9] overflow-hidden text-[#202733] select-none font-sans border border-[#E2E7EF] shadow-window">
      {/* 1. Custom Title Bar */}
      <TitleBar />

      {/* 2. Main Workspace (Sidebar + Dynamic Workspace Content + Blocking Loading Overlay) */}
      <div className="relative flex flex-1 min-h-0 min-w-0 overflow-hidden">
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

        {/* 3. Dashboard Initialization & Hibernation Loading Overlay */}
        <DashboardLoadingOverlay />
      </div>

      {/* 4. Dynamic Bubble Spawn Directional Arrow Indicator */}
      <BubbleSpawnIndicator />
    </div>
  );
};

export default App;
