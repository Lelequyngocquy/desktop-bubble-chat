import React from "react";
import { Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TabHeader } from "../common/TabHeader";
import {
  usePerformanceStore,
  usePerformanceAutoPoll,
} from "../../stores/performanceStore";
import { PerformanceSummaryCards } from "./PerformanceSummaryCards";
import { MemoryDistributionBar } from "./MemoryDistributionBar";
import { ProcessDetailCard } from "./ProcessDetailCard";

export const PerformanceWorkspace: React.FC = () => {
  const { t } = useTranslation("performance");
  usePerformanceAutoPoll();
  const snapshot = usePerformanceStore((s) => s.snapshot);
  const isLoading = usePerformanceStore((s) => s.isLoading);
  const isTrimming = usePerformanceStore((s) => s.isTrimming);
  const lastTrimmedMb = usePerformanceStore((s) => s.lastTrimmedMb);
  const fetchSnapshot = usePerformanceStore((s) => s.fetchSnapshot);
  const trimMemory = usePerformanceStore((s) => s.trimMemory);

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
            <button
              onClick={() => trimMemory()}
              disabled={isTrimming}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-hover text-white rounded-pill text-caption font-semibold shadow-xs transition-colors cursor-pointer disabled:opacity-60"
              title={t("trim_ram_tooltip")}
            >
              <Sparkles className={`w-3.5 h-3.5 ${isTrimming ? "animate-spin" : ""}`} />
              <span>{isTrimming ? t("trim_ram_reclaiming") : t("trim_ram_btn")}</span>
            </button>
          </div>
        }
      />
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <PerformanceSummaryCards snapshot={snapshot} isLoading={isLoading} />
        <MemoryDistributionBar snapshot={snapshot} />
        <ProcessDetailCard
          snapshot={snapshot}
          isLoading={isLoading}
          isTrimming={isTrimming}
          lastTrimmedMb={lastTrimmedMb}
          onRefresh={fetchSnapshot}
          onTrimMemory={trimMemory}
        />
      </main>
    </div>
  );
};

export default PerformanceWorkspace;