import React from "react";
import { Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TabHeader } from "../common/TabHeader";
import {
  usePerformanceStore,
  usePerformanceAutoPoll,
} from "../../stores/performanceStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { PerformanceSummaryCards } from "./PerformanceSummaryCards";
import { MemoryDistributionBar } from "./MemoryDistributionBar";
import { ProcessDetailCard } from "./ProcessDetailCard";

export const PerformanceWorkspace: React.FC = () => {
  const { t } = useTranslation("performance");
  usePerformanceAutoPoll();
  const snapshot = usePerformanceStore((s) => s.snapshot);
  const isLoading = usePerformanceStore((s) => s.isLoading);
  const isAutoTrimEnabled = useSettingsStore((s) => s.isAutoTrimEnabled);

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      <TabHeader
        eyebrow={t("header_eyebrow")}
        title={t("header_title")}
        description={t("header_desc")}
        rightSlot={
          <div className="flex items-center gap-2.5">
            <div className="flex items-center gap-2 px-3 py-1.5 bg-background-surface border border-border rounded-pill text-small shadow-xs">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
              <span className="text-content-primary font-medium text-caption">
                {t("realtime_badge")}
              </span>
            </div>
            <div
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-pill text-caption font-semibold shadow-xs border ${
                isAutoTrimEnabled
                  ? "bg-primary/10 border-primary text-primary"
                  : "bg-background-surface border-border text-content-secondary"
              }`}
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>
                {isAutoTrimEnabled
                  ? t("auto_trim_active")
                  : t("auto_trim_disabled")}
              </span>
            </div>
          </div>
        }
      />
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <PerformanceSummaryCards snapshot={snapshot} isLoading={isLoading} />
        <MemoryDistributionBar snapshot={snapshot} />
        <ProcessDetailCard snapshot={snapshot} isLoading={isLoading} />
      </main>
    </div>
  );
};

export default PerformanceWorkspace;