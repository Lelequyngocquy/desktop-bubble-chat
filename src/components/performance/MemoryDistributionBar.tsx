import React, { useMemo } from "react";
import { PieChart } from "lucide-react";
import { useTranslation } from "react-i18next";
import { SystemPerformanceSnapshot } from "../../types/performance";

interface MemoryDistributionBarProps {
  snapshot: SystemPerformanceSnapshot | null;
  isLoading: boolean;
}

export const MemoryDistributionBar: React.FC<MemoryDistributionBarProps> = React.memo(({
  snapshot,
  isLoading,
}) => {
  const { t } = useTranslation("performance");
  const { hasSnapshot, totalRam, segments } = useMemo(() => {
    const valid = Boolean(snapshot);
    const total = snapshot?.totalRamMb ?? 0;
    const divisor = total > 0 ? total : 1;
    const hostRam = snapshot?.hostRamMb ?? 0;
    const dashboardRam = snapshot?.dashboardRamMb ?? 0;
    let messengerRam = 0;
    let gpuUtilityRam = 0;
    if (snapshot?.items) {
      for (const item of snapshot.items) {
        if (item.category === "messenger") {
          messengerRam += item.workingSetMb;
        } else if (item.category === "gpu" || item.category === "utility") {
          gpuUtilityRam += item.workingSetMb;
        }
      }
    }
    const items = [
      {
        key: "host",
        label: t("segment_host"),
        ramMb: hostRam,
        pct: valid ? Math.max(0, (hostRam / divisor) * 100) : 0,
        colorClass: "bg-[#0866FF]",
        dotClass: "bg-[#0866FF]",
      },
      {
        key: "dashboard",
        label: t("segment_dashboard"),
        ramMb: dashboardRam,
        pct: valid ? Math.max(0, (dashboardRam / divisor) * 100) : 0,
        colorClass: "bg-[#8B5CF6]",
        dotClass: "bg-[#8B5CF6]",
      },
      {
        key: "messenger",
        label: t("segment_messenger"),
        ramMb: messengerRam,
        pct: valid ? Math.max(0, (messengerRam / divisor) * 100) : 0,
        colorClass: "bg-[#00C6FF]",
        dotClass: "bg-[#00C6FF]",
      },
      {
        key: "gpu",
        label: t("segment_gpu"),
        ramMb: gpuUtilityRam,
        pct: valid ? Math.max(0, (gpuUtilityRam / divisor) * 100) : 0,
        colorClass: "bg-[#10B981]",
        dotClass: "bg-[#10B981]",
      },
    ];
    return { hasSnapshot: valid, totalRam: total, segments: items };
  }, [snapshot, t]);
  return (
    <div className="bg-background-surface border border-border rounded-panel p-5 shadow-xs flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <PieChart className="w-4 h-4 text-primary" />
          <h3 className="text-body font-semibold text-content-primary m-0">
            {t("memory_distribution_title")}
          </h3>
        </div>
        <span className="text-caption text-content-muted">
          {hasSnapshot
            ? `${t("total_label")}: ${Math.round(totalRam)} MB (100%)`
            : isLoading
              ? t("refresh_running")
              : `${t("total_label")}: 0 MB (0%)`}
        </span>
      </div>
      <div className="w-full h-3.5 bg-slate-100 dark:bg-background-app rounded-pill overflow-hidden flex p-0.5 border border-border gap-0.5">
        {segments.map((seg) => {
          if (seg.pct <= 0) return null;
          return (
            <div
              key={seg.key}
              className={`h-full rounded-pill transition-all duration-500 ${seg.colorClass}`}
              style={{ width: `${seg.pct}%` }}
              title={`${seg.label}: ${seg.ramMb.toFixed(1)} MB (${seg.pct.toFixed(1)}%)`}
            />
          );
        })}
      </div>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 pt-1">
        {segments.map((seg) => (
          <div key={seg.key} className="flex flex-col gap-0.5">
            <div className="flex items-center gap-1.5 text-caption font-medium text-content-secondary">
              <span className={`w-2 h-2 rounded-full shrink-0 ${seg.dotClass}`} />
              <span className="truncate">{seg.label}</span>
            </div>
            <div className="flex items-baseline gap-1.5 pl-3.5">
              <span className="text-small font-semibold text-content-primary">
                {hasSnapshot ? `${seg.ramMb.toFixed(1)} MB` : "--"}
              </span>
              <span className="text-caption text-content-muted">
                ({seg.pct.toFixed(1)}%)
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
});

export default MemoryDistributionBar;
