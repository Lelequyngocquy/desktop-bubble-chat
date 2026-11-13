import React from "react";
import { Lightbulb, MonitorCheck } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useSystemUserStore } from "../../stores/systemUserStore";
import { TabHeader } from "../common/TabHeader";
import {
  MessengerConnectionCard,
  ActiveChatHeads,
  QuickSettings,
} from "./components";
import { CommonHintLine } from "../common";

export const HomeWorkspace: React.FC = () => {
  const { t } = useTranslation("dashboard");
  const systemUser = useSystemUserStore((s) => s.systemUser);
  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden">
      <TabHeader
        eyebrow={t("workspace_eyebrow")}
        title={t("welcome_user", { user: systemUser })}
        description={t("workspace_desc")}
        rightSlot={
          <div className="flex items-center gap-2 px-3 py-1.5 bg-background-surface border border-border rounded-pill shadow-xs">
            <MonitorCheck className="w-3.5 h-3.5 text-status-success" />
            <span className="text-small text-content-primary font-medium">
              {t("ready_desktop")}
            </span>
          </div>
        }
      />
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <MessengerConnectionCard />
        <div className="shrink-0 flex flex-col lg:flex-row items-stretch gap-6 h-auto lg:h-side-col">
          <ActiveChatHeads />
          <QuickSettings />
        </div>
        <CommonHintLine
          icon={<Lightbulb />}
          text={t("footer_tip")}
          className="bg-transparent p-0 rounded-none"
        />
      </main>
    </div>
  );
};

export default HomeWorkspace;
