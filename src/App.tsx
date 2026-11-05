import React from "react";
import { TitleBar } from "./components/layout/TitleBar";
import { Sidebar } from "./components/layout/Sidebar";
import { HomeWorkspace } from "./components/home";
import { ConnectionsWorkspace } from "./components/connections";
import { NotificationsWorkspace } from "./components/notifications";
import { PerformanceWorkspace } from "./components/performance";
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
  // Initialize listeners, bootstrap lifecycle, and auto-restore background bubbles
  useDashboardBootstrap();
  useUnreadListener();
  useSystemUser();
  useBubbleAutoRestore();
  useAutoUpdater();
  return (
    <div className="flex flex-col h-screen w-screen bg-[#EEF2F9] overflow-hidden text-[#202733] select-none font-sans border border-[#E2E7EF] shadow-window">
      <TitleBar />
      <div className="relative flex flex-1 min-h-0 min-w-0 overflow-hidden">
        <Sidebar />
        {activeTab === "connections" ? (
          <ConnectionsWorkspace />
        ) : activeTab === "notifications" || (activeTab as string) === "chat-heads" ? (
          <NotificationsWorkspace />
        ) : activeTab === "performance" ? (
          <PerformanceWorkspace />
        ) : activeTab === "settings" ? (
          <SettingsWorkspace />
        ) : activeTab === "help" ? (
          <HelpWorkspace />
        ) : (
          <HomeWorkspace />
        )}
        <DashboardLoadingOverlay />
      </div>
      <BubbleSpawnIndicator />
    </div>
  );
};

export default App;
