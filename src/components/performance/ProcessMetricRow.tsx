import React from "react";

export interface ProcessMetricRowProps {
  name: string;
  pid: number;
  subtitle?: string;
  workingSetMb: number;
  ramLabel?: string;
  cpuPercent: number;
  cpuLabel: string;
  variant?: "default" | "compact";
}

export const ProcessMetricRow: React.FC<ProcessMetricRowProps> = ({
  name,
  pid,
  subtitle,
  workingSetMb,
  ramLabel,
  cpuPercent,
  cpuLabel,
  variant = "default",
}) => {
  if (variant === "compact") {
    return (
      <div className="p-2.5 bg-background-app/40 border border-border rounded-control flex items-center justify-between text-caption">
        <div className="flex flex-col">
          <span className="font-medium text-content-primary">{name}</span>
          <span className="text-content-muted">PID: {pid}</span>
        </div>
        <div className="flex items-center gap-3">
          <span className="font-semibold text-content-primary">
            {workingSetMb.toFixed(1)} MB
          </span>
          <span className="text-content-secondary">
            {cpuPercent.toFixed(1)}% {cpuLabel}
          </span>
        </div>
      </div>
    );
  }
  return (
    <div className="p-3 bg-background-app/50 border border-border rounded-control flex flex-col sm:flex-row sm:items-center justify-between gap-2">
      <div className="flex flex-col gap-0.5">
        <span className="text-small font-medium text-content-primary">{name}</span>
        <span className="text-caption text-content-muted">
          PID: {pid}
          {subtitle ? ` · ${subtitle}` : ""}
        </span>
      </div>
      <div className="flex items-center gap-4 text-small">
        <div className="flex flex-col items-end">
          <span className="font-semibold text-content-primary">
            {workingSetMb.toFixed(1)} MB
          </span>
          {ramLabel && (
            <span className="text-caption text-content-secondary">{ramLabel}</span>
          )}
        </div>
        <div className="flex flex-col items-end w-14">
          <span className="font-semibold text-content-primary">
            {cpuPercent.toFixed(1)}%
          </span>
          <span className="text-caption text-content-secondary">{cpuLabel}</span>
        </div>
      </div>
    </div>
  );
};
