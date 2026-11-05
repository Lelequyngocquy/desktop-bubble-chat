import React from "react";
import { CheckCircle2, AlertCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  useMessengerStore,
  selectIsMessengerConnected,
} from "../../platforms/messenger";
import { TabHeader } from "../common/TabHeader";
import { MessengerManagerCard, MessengerTroubleshoot } from "./components";
import { PlatformRoadmap } from "../home";

export const ConnectionsWorkspace: React.FC = () => {
  const { t } = useTranslation("dashboard");
  const isMessengerConnected = useMessengerStore(selectIsMessengerConnected);
  const accounts = useMessengerStore((s) => s.accounts);
  const connectedCount = accounts.filter((a) => a.isConnected).length;

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      <TabHeader
        eyebrow={t("workspaces.connections.eyebrow")}
        title={t("workspaces.connections.title")}
        description={t("workspaces.connections.description")}
        rightSlot={
          <div className="flex items-center gap-1.5 px-3 py-1.5 bg-background-surface border border-border rounded-pill text-small shadow-xs">
            {isMessengerConnected ? (
              <>
                <CheckCircle2 className="w-3.5 h-3.5 text-status-success" />
                <span className="text-content-primary font-medium">
                  {connectedCount > 1
                    ? t("workspaces.connections.connected_count_plural", {
                        count: connectedCount,
                      })
                    : t("workspaces.connections.connected_count_single")}
                </span>
              </>
            ) : (
              <>
                <AlertCircle className="w-3.5 h-3.5 text-status-warning" />
                <span className="text-content-secondary font-medium">
                  {t("workspaces.connections.not_connected")}
                </span>
              </>
            )}
          </div>
        }
      />
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <div className="shrink-0 flex flex-col xl:flex-row items-stretch gap-6">
          <MessengerManagerCard />
        </div>
        <MessengerTroubleshoot />
        <PlatformRoadmap />
      </main>
    </div>
  );
};

export default ConnectionsWorkspace;
