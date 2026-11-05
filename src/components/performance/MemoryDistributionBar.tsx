import React from "react";
import { PieChart } from "lucide-react";
import { useTranslation } from "react-i18next";
import { SystemPerformanceSnapshot } from "../../types/performance";

interface MemoryDistributionBarProps {
  snapshot: SystemPerformanceSnapshot | null;
}

export const MemoryDistributionBar: React.FC<MemoryDistributionBarProps> = ({
  snapshot,
}) => {
  const { t } = useTranslation("performance");
  const totalRam = snapshot?.totalRamMb || 1;
  const hostRam = snapshot?.hostRamMb || 0;
  const dashboardRam = snapshot?.dashboardRamMb || 0;
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
  const hostPct = Math.max(0, (hostRam / totalRam) * 100);
  const dashPct = Math.max(0, (dashboardRam / totalRam) * 100);
  const msgPct = Math.max(0, (messengerRam / totalRam) * 100);
  const gpuPct = Math.max(0, (gpuUtilityRam / totalRam) * 100);
  const segments = [
    {
      label: t("segment_host"),
      ramMb: hostRam,
      pct: hostPct,
      colorClass: "bg-[#0866FF]",
      textClass: "text-[#0866FF]",
      dotClass: "bg-[#0866FF]",
    },
    {
      label: t("segment_dashboard"),
      ramMb: dashboardRam,
      pct: dashPct,
      colorClass: "bg-[#8B5CF6]",
      textClass: "text-[#8B5CF6]",
      dotClass: "bg-[#8B5CF6]",
    },
    {
      label: t("segment_messenger"),
      ramMb: messengerRam,
      pct: msgPct,
      colorClass: "bg-[#00C6FF]",
      textClass: "text-[#0080B0]",
      dotClass: "bg-[#00C6FF]",
    },
    {
      label: t("segment_gpu"),
      ramMb: gpuUtilityRam,
      pct: gpuPct,
      colorClass: "bg-[#10B981]",
      textClass: "text-[#059669]",
      dotClass: "bg-[#10B981]",
    },
  ];

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
          {t("total_label")}: {Math.round(totalRam)} MB (100%)
        </span>
      </div>
      <div className="w-full h-3.5 bg-slate-100 dark:bg-background-app rounded-pill overflow-hidden flex p-0.5 border border-border gap-0.5">
        {segments.map((seg, idx) => {
          if (seg.pct <= 0) return null;
          return (
            <div
              key={idx}
              className={`h-full rounded-pill transition-all duration-500 ${seg.colorClass}`}
              style={{ width: `${seg.pct}%` }}
              title={`${seg.label}: ${seg.ramMb.toFixed(1)} MB (${seg.pct.toFixed(1)}%)`}
            />
          );
        })}
      </div>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 pt-1">
        {segments.map((seg, idx) => (
          <div key={idx} className="flex flex-col gap-0.5">
            <div className="flex items-center gap-1.5 text-caption font-medium text-content-secondary">
              <span className={`w-2 h-2 rounded-full shrink-0 ${seg.dotClass}`} />
              <span className="truncate">{seg.label}</span>
            </div>
            <div className="flex items-baseline gap-1.5 pl-3.5">
              <span className="text-small font-semibold text-content-primary">
                {seg.ramMb.toFixed(1)} MB
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
};

export default MemoryDistributionBar;