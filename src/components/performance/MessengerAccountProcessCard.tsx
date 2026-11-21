import React from "react";
import { Snowflake, Play, CheckCircle2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ProcessMetricItem } from "../../types/performance";
import { CommonBadge } from "../common";
import { cn } from "../../utils/cn";

export interface MessengerAccountProcessCardProps {
  accountIndex: number;
  procList: ProcessMetricItem[];
  isActing: boolean;
  onFreeze: (accountIndex: number) => void;
  onWake: (accountIndex: number) => void;
}

export const MessengerAccountProcessCard: React.FC<MessengerAccountProcessCardProps> = ({
  accountIndex,
  procList,
  isActing,
  onFreeze,
  onWake,
}) => {
  const { t } = useTranslation("performance");
  const isSuspended = procList.every((p) => p.isSuspended);
  const totalAccountRam = procList.reduce((sum, p) => sum + p.workingSetMb, 0);
  const totalAccountCpu = procList.reduce((sum, p) => sum + p.cpuPercent, 0);
  const paddedIndex = accountIndex.toString().padStart(2, "0");
  const displayName = procList[0]?.accountName || `Messenger ${paddedIndex}`;
  return (
    <div
      className={cn(
        "p-3.5 border rounded-panel flex flex-col gap-3 transition-colors",
        isSuspended
          ? "bg-background-subtle border-border"
          : "bg-background-surface border-border shadow-xs",
      )}
    >
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5">
        <div className="flex items-center gap-2.5">
          <div
            className={cn(
              "w-9 h-9 rounded-xl flex items-center justify-center font-bold text-small shrink-0",
              isSuspended
                ? "bg-background-app text-content-muted"
                : "bg-gradient-to-br from-blue-500 to-sky-400 text-white shadow-xs",
            )}
          >
            {paddedIndex}
          </div>
          <div className="flex flex-col">
            <div className="flex items-center gap-2">
              <span className="text-small font-semibold text-content-primary">
                {displayName}
              </span>
              {isSuspended ? (
                <CommonBadge
                  status={t("status_frozen")}
                  color="secondary"
                  icon={<Snowflake className="w-3 h-3" />}
                />
              ) : (
                <CommonBadge
                  status={t("status_active")}
                  color="success"
                  icon={<CheckCircle2 className="w-3 h-3" />}
                />
              )}
            </div>
            <span className="text-caption text-content-muted">
              {t("profile_dir")}: profiles/messenger_{paddedIndex}
            </span>
          </div>
        </div>
        <div className="flex items-center gap-3 self-end sm:self-auto">
          <div className="flex items-center gap-3 text-small mr-2">
            <div className="flex flex-col items-end">
              <span
                className={cn(
                  "font-semibold",
                  isSuspended ? "text-emerald-600 font-bold" : "text-content-primary",
                )}
              >
                {isSuspended ? "0 MB" : `${totalAccountRam.toFixed(1)} MB`}
              </span>
              <span className="text-caption text-content-muted">
                {isSuspended ? t("savings_tag") : t("total_ram")}
              </span>
            </div>
            <div className="flex flex-col items-end w-12">
              <span className="font-semibold text-content-primary">
                {isSuspended ? "0.0%" : `${totalAccountCpu.toFixed(1)}%`}
              </span>
              <span className="text-caption text-content-muted">{t("cpu")}</span>
            </div>
          </div>
          {isSuspended ? (
            <button
              type="button"
              onClick={() => onWake(accountIndex)}
              disabled={isActing}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-foreground text-white rounded-control text-small font-medium shadow-xs transition-colors cursor-pointer disabled:opacity-60"
              title={t("wake_tooltip")}
            >
              <Play className={cn("w-3.5 h-3.5 fill-current", isActing && "animate-spin")} />
              <span>{isActing ? t("wake_running") : t("wake_account")}</span>
            </button>
          ) : (
            <button
              type="button"
              onClick={() => onFreeze(accountIndex)}
              disabled={isActing}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-subtle hover:bg-primary/20 text-primary-foreground border border-primary/25 rounded-control text-small font-medium transition-colors cursor-pointer disabled:opacity-60"
              title={t("freeze_tooltip")}
            >
              <Snowflake className={cn("w-3.5 h-3.5", isActing && "animate-spin")} />
              <span>{isActing ? t("freeze_running") : t("freeze_account")}</span>
            </button>
          )}
        </div>
      </div>
      {!isSuspended && procList.length > 0 && (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-2 border-t border-border">
          {procList.map((p) => (
            <div
              key={p.pid}
              className="px-2.5 py-1.5 bg-background-app/50 rounded-control text-caption flex items-center gap-2"
            >
              <span
                className="w-[70%] min-w-0 truncate text-content-secondary"
                title={`${p.name} (PID: ${p.pid})`}
              >
                {p.name} (PID: {p.pid})
              </span>
              <span className="w-[30%] shrink-0 text-right font-semibold text-content-primary whitespace-nowrap">
                {p.workingSetMb.toFixed(1)} MB · {p.cpuPercent.toFixed(1)}%
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
