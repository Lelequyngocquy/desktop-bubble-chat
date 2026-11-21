import React, { Suspense, lazy } from "react";
import { TitleBar } from "./components/layout/TitleBar";
import { Sidebar } from "./components/layout/Sidebar";
import { HomeWorkspace } from "./components/home";
import { BubbleSpawnIndicator } from "./components/common/BubbleSpawnIndicator";
import { DashboardLoadingOverlay } from "./components/common/DashboardLoadingOverlay";
import { useDashboardBootstrap, useAutoUpdaterLifecycle, useThemeSync } from "./hooks";
import { useUnreadListener } from "./hooks/useUnreadListener";
import { useNavigationStore } from "./stores/navigationStore";

const ConnectionsWorkspace = lazy(() => import("./components/connections").then((m) => ({ default: m.ConnectionsWorkspace })));
const NotificationsWorkspace = lazy(() => import("./components/notifications").then((m) => ({ default: m.NotificationsWorkspace })));
const PerformanceWorkspace = lazy(() => import("./components/performance").then((m) => ({ default: m.PerformanceWorkspace })));
const SettingsWorkspace = lazy(() => import("./components/settings").then((m) => ({ default: m.SettingsWorkspace })));
const HelpWorkspace = lazy(() => import("./components/help").then((m) => ({ default: m.HelpWorkspace })));

export const App: React.FC = () => {
  const activeTab = useNavigationStore((s) => s.activeTab);
  // Initialize listeners, unified bootstrap lifecycle, and theme sync
  useDashboardBootstrap();
  useThemeSync();
  useUnreadListener();
  useAutoUpdaterLifecycle();
  return (
    <div className="flex flex-col h-screen w-screen bg-background-app overflow-hidden text-content-primary select-none font-sans border border-border shadow-window">
      <TitleBar />
      <div className="relative flex flex-1 min-h-0 min-w-0 overflow-hidden">
        <Sidebar />
        <Suspense fallback={<DashboardLoadingOverlay />}>
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
        </Suspense>
        <DashboardLoadingOverlay />
      </div>
      <BubbleSpawnIndicator />
    </div>
  );
};

export default App;
