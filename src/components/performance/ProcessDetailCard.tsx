import React, { useState, useRef, useEffect, useMemo } from "react";
import {
  Layers,
  Snowflake,
  Cpu,
  Monitor,
  HardDrive,
  Info,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  ProcessMetricItem,
  SystemPerformanceSnapshot,
} from "../../types/performance";
import { messengerService } from "../../platforms/messenger/services/messengerService";
import { usePerformanceStore } from "../../stores/performanceStore";
import { ProcessMetricRow } from "./ProcessMetricRow";
import { MessengerAccountProcessCard } from "./MessengerAccountProcessCard";

const getHostEngineLabel = () => {
  const ua = typeof navigator !== "undefined" ? navigator.userAgent.toLowerCase() : "";
  if (ua.includes("linux")) return "GTK3 / X11 Event Loop & Skia Vector Engine";
  if (ua.includes("mac")) return "Cocoa RunLoop & Skia Vector Engine";
  return "Win32 Message Loop & Skia Vector Engine";
};

interface ProcessDetailCardProps {
  snapshot: SystemPerformanceSnapshot | null;
  isLoading: boolean;
}

export const ProcessDetailCard: React.FC<ProcessDetailCardProps> = React.memo(({
  snapshot,
  isLoading,
}) => {
  const { t } = useTranslation("performance");
  const [actingAccountIndices, setActingAccountIndices] = useState<Set<number>>(
    () => new Set(),
  );
  const timersRef = useRef<Set<ReturnType<typeof setTimeout>>>(new Set());
  const isMountedRef = useRef(true);
  useEffect(() => {
    isMountedRef.current = true;
    const timers = timersRef.current;
    return () => {
      isMountedRef.current = false;
      timers.forEach((timer) => clearTimeout(timer));
      timers.clear();
    };
  }, []);
  const setAccountActing = (accountIndex: number, acting: boolean) => {
    if (!isMountedRef.current) return;
    setActingAccountIndices((prev) => {
      const next = new Set(prev);
      if (acting) {
        next.add(accountIndex);
      } else {
        next.delete(accountIndex);
      }
      return next;
    });
  };
  const { hostItems, dashboardItems, messengerAccounts, gpuUtilityItems } = useMemo(() => {
    const host: ProcessMetricItem[] = [];
    const dashboard: ProcessMetricItem[] = [];
    const messengerMap = new Map<number, ProcessMetricItem[]>();
    const gpuUtility: ProcessMetricItem[] = [];
    if (snapshot?.items) {
      for (const item of snapshot.items) {
        if (item.category === "host") {
          host.push(item);
        } else if (item.category === "dashboard") {
          dashboard.push(item);
        } else if (item.category === "messenger" && item.accountIndex) {
          const list = messengerMap.get(item.accountIndex) || [];
          list.push(item);
          messengerMap.set(item.accountIndex, list);
        } else {
          gpuUtility.push(item);
        }
      }
    }
    return {
      hostItems: host,
      dashboardItems: dashboard,
      messengerAccounts: Array.from(messengerMap.entries()).sort(([a], [b]) => a - b),
      gpuUtilityItems: gpuUtility,
    };
  }, [snapshot]);
  const scheduleRefreshAndClear = (accountIndex: number, delayMs: number) => {
    const timer = setTimeout(() => {
      timersRef.current.delete(timer);
      if (!isMountedRef.current) return;
      usePerformanceStore.getState().fetchSnapshot();
      setAccountActing(accountIndex, false);
    }, delayMs);
    timersRef.current.add(timer);
  };
  const handleFreezeAccount = async (accountIndex: number) => {
    try {
      setAccountActing(accountIndex, true);
      await messengerService.toggleBubble(false, accountIndex);
      if (!isMountedRef.current) return;
      scheduleRefreshAndClear(accountIndex, 500);
    } catch (err) {
      console.error("[ProcessDetailCard] Error freezing account:", err);
      setAccountActing(accountIndex, false);
    }
  };
  const handleWakeAccount = async (accountIndex: number) => {
    try {
      setAccountActing(accountIndex, true);
      await messengerService.openChat(accountIndex);
      if (!isMountedRef.current) return;
      scheduleRefreshAndClear(accountIndex, 800);
    } catch (err) {
      console.error("[ProcessDetailCard] Error waking account:", err);
      setAccountActing(accountIndex, false);
    }
  };
  const hostEngine = getHostEngineLabel();
  return (
    <div className="bg-background-surface border border-border rounded-panel p-5 shadow-xs flex flex-col gap-5">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-border">
        <div className="flex items-center gap-2">
          <Layers className="w-4 h-4 text-primary" />
          <h2 className="text-body-lg font-semibold text-content-primary m-0">
            {t("title")}
          </h2>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-1.5 px-2.5 py-1 bg-background-workspace border border-border rounded-pill text-caption text-content-secondary shadow-2xs">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
            <span>{t("realtime_badge")}</span>
          </div>
        </div>
      </div>
      <div className="flex flex-col gap-4 divide-y divide-border">
        <div className="flex flex-col gap-2 pt-1 first:pt-0">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-background-subtle text-primary flex items-center justify-center">
                <Cpu className="w-3.5 h-3.5" />
              </div>
              <span className="text-small font-semibold text-content-primary">
                {t("host_core")}
              </span>
            </div>
            <span className="text-caption text-emerald-600 font-medium flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
              {t("optimal_status")}
            </span>
          </div>
          {hostItems.map((item) => (
            <ProcessMetricRow
              key={item.pid}
              name={item.name}
              pid={item.pid}
              subtitle={hostEngine}
              workingSetMb={item.workingSetMb}
              ramLabel={t("ram_working_set")}
              cpuPercent={item.cpuPercent}
              cpuLabel={t("cpu")}
            />
          ))}
        </div>
        <div className="flex flex-col gap-2 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-background-subtle text-purple-500 flex items-center justify-center">
                <Monitor className="w-3.5 h-3.5" />
              </div>
              <span className="text-small font-semibold text-content-primary">
                {t("dashboard_ui")}
              </span>
            </div>
            <span className="text-caption text-content-muted flex items-center gap-1">
              <Info className="w-3.5 h-3.5" />
              {t("dashboard_hibernation_hint")}
            </span>
          </div>
          {dashboardItems.length === 0 ? (
            <div className="p-3 bg-background-app/30 border border-border rounded-control text-caption text-content-muted">
              {!snapshot && isLoading ? t("refresh_running") : t("dashboard_hibernating")}
            </div>
          ) : (
            dashboardItems.map((item) => (
              <ProcessMetricRow
                key={item.pid}
                name={item.name}
                pid={item.pid}
                subtitle="React 19 Dashboard UI"
                workingSetMb={item.workingSetMb}
                ramLabel={t("ram")}
                cpuPercent={item.cpuPercent}
                cpuLabel={t("cpu")}
              />
            ))
          )}
        </div>
        <div className="flex flex-col gap-2.5 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-background-subtle text-sky-500 flex items-center justify-center">
                <Snowflake className="w-3.5 h-3.5" />
              </div>
              <span className="text-small font-semibold text-content-primary">
                {t("messenger_accounts")}
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              {t("isolated_webview_hint")}
            </span>
          </div>
          {messengerAccounts.length === 0 ? (
            <div className="p-4 bg-background-app/30 border border-border rounded-control text-center text-caption text-content-muted">
              {t("no_accounts_linked")}
            </div>
          ) : (
            messengerAccounts.map(([accountIndex, procList]) => (
              <MessengerAccountProcessCard
                key={accountIndex}
                accountIndex={accountIndex}
                procList={procList}
                isActing={actingAccountIndices.has(accountIndex)}
                onFreeze={handleFreezeAccount}
                onWake={handleWakeAccount}
              />
            ))
          )}
        </div>
        <div className="flex flex-col gap-2 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-background-subtle text-emerald-500 flex items-center justify-center">
                <HardDrive className="w-3.5 h-3.5" />
              </div>
              <span className="text-small font-semibold text-content-primary">
                {t("gpu_utility")}
              </span>
            </div>
            <span className="text-caption text-content-muted">
              {t("shared_runtime")}
            </span>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {gpuUtilityItems.map((item) => (
              <ProcessMetricRow
                key={item.pid}
                name={item.name}
                pid={item.pid}
                workingSetMb={item.workingSetMb}
                cpuPercent={item.cpuPercent}
                cpuLabel={t("cpu")}
                variant="compact"
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
});

export default ProcessDetailCard;
