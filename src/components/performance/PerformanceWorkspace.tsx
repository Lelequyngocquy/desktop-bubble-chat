import React from "react";
import { AlertCircle, RefreshCw, Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TabHeader } from "../common/TabHeader";
import { usePerformanceStore } from "../../stores/performanceStore";
import { usePerformanceAutoPoll } from "../../hooks";
import { useSettingsStore } from "../../stores/settingsStore";
import { PerformanceSummaryCards } from "./PerformanceSummaryCards";
import { MemoryDistributionBar } from "./MemoryDistributionBar";
import { ProcessDetailCard } from "./ProcessDetailCard";

export const PerformanceWorkspace: React.FC = () => {
  const { t } = useTranslation(["performance", "common"]);
  usePerformanceAutoPoll();
  const snapshot = usePerformanceStore((s) => s.snapshot);
  const isLoading = usePerformanceStore((s) => s.isLoading);
  const error = usePerformanceStore((s) => s.error);
  const isAutoTrimEnabled = useSettingsStore((s) => s.isAutoTrimEnabled);
  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      <TabHeader
        title={t("performance:header_title")}
        description={t("performance:header_desc")}
        rightSlot={
          <div className="flex items-center gap-2.5">
            <div className="flex items-center gap-2 px-3 py-1.5 bg-background-surface border border-border rounded-pill text-small shadow-xs">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
              <span className="text-content-primary font-medium text-caption">
                {t("performance:realtime_badge")}
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
                  ? t("performance:auto_trim_active")
                  : t("performance:auto_trim_disabled")}
              </span>
            </div>
          </div>
        }
      />
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {error && (
          <div className="p-3.5 bg-status-errorBg border border-status-errorBorder rounded-panel text-status-error text-small flex items-center justify-between gap-3 shrink-0">
            <div className="flex items-center gap-2 min-w-0">
              <AlertCircle className="w-4 h-4 shrink-0" />
              <span className="truncate">{error}</span>
            </div>
            <button
              type="button"
              onClick={() => usePerformanceStore.getState().fetchSnapshot()}
              disabled={isLoading}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded-control bg-background-surface border border-status-errorBorder text-caption font-semibold hover:bg-background-subtle transition-colors cursor-pointer shrink-0 disabled:opacity-60"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? "animate-spin" : ""}`} />
              <span>{t("common:retry")}</span>
            </button>
          </div>
        )}
        <PerformanceSummaryCards snapshot={snapshot} isLoading={isLoading} />
        <MemoryDistributionBar snapshot={snapshot} isLoading={isLoading} />
        <ProcessDetailCard snapshot={snapshot} isLoading={isLoading} />
      </main>
    </div>
  );
};

export default PerformanceWorkspace;
