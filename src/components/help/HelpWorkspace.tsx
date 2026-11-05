import React from "react";
import { useTranslation } from "react-i18next";
import { TabHeader } from "../common";
import {
  QuickTipsSection,
  ResourceOptimizationCard,
  FeedbackSection,
  MessengerExplainerCard,
} from "./components";

export const HelpWorkspace: React.FC = () => {
  const { t } = useTranslation("help");

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      <TabHeader
        eyebrow={t("header_eyebrow")}
        title={t("header_title")}
        description={t("header_desc")}
      />
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <QuickTipsSection />
        <ResourceOptimizationCard />
        <MessengerExplainerCard />
        <FeedbackSection />
      </main>
    </div>
  );
};

export default HelpWorkspace;
