import React, { useState } from "react";
import {
  Layers,
  Snowflake,
  Play,
  CheckCircle2,
  Cpu,
  Monitor,
  HardDrive,
  Info,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import {
  ProcessMetricItem,
  SystemPerformanceSnapshot,
} from "../../types/performance";
import { IPC_COMMANDS } from "../../types/ipc";
import { CommonBadge } from "../common";
import { usePerformanceStore } from "../../stores/performanceStore";

interface ProcessDetailCardProps {
  snapshot: SystemPerformanceSnapshot | null;
  isLoading: boolean;
}

export const ProcessDetailCard: React.FC<ProcessDetailCardProps> = ({
  snapshot,
  isLoading: _isLoading,
}) => {
  const { t } = useTranslation("performance");
  const [actingAccountIndex, setActingAccountIndex] = useState<number | null>(
    null,
  );
  const hostItems: ProcessMetricItem[] = [];
  const dashboardItems: ProcessMetricItem[] = [];
  const messengerItemsMap = new Map<number, ProcessMetricItem[]>();
  const gpuUtilityItems: ProcessMetricItem[] = [];
  if (snapshot?.items) {
    for (const item of snapshot.items) {
      if (item.category === "host") {
        hostItems.push(item);
      } else if (item.category === "dashboard") {
        dashboardItems.push(item);
      } else if (item.category === "messenger" && item.accountIndex) {
        const list = messengerItemsMap.get(item.accountIndex) || [];
        list.push(item);
        messengerItemsMap.set(item.accountIndex, list);
      } else {
        gpuUtilityItems.push(item);
      }
    }
  }
  const messengerAccounts = Array.from(messengerItemsMap.entries()).sort(
    ([a], [b]) => a - b,
  );
  const handleFreezeAccount = async (accountIndex: number) => {
    try {
      setActingAccountIndex(accountIndex);
      await invoke(IPC_COMMANDS.TOGGLE_BUBBLE, {
        accountIndex,
        visible: false,
      });
      setTimeout(() => {
        usePerformanceStore.getState().fetchSnapshot();
        setActingAccountIndex(null);
      }, 500);
    } catch (err) {
      console.error("[ProcessDetailCard] Error freezing account:", err);
      setActingAccountIndex(null);
    }
  };
  const handleWakeAccount = async (accountIndex: number) => {
    try {
      setActingAccountIndex(accountIndex);
      await invoke(IPC_COMMANDS.OPEN_CHAT, { accountIndex });
      setTimeout(() => {
        usePerformanceStore.getState().fetchSnapshot();
        setActingAccountIndex(null);
      }, 800);
    } catch (err) {
      console.error("[ProcessDetailCard] Error waking account:", err);
      setActingAccountIndex(null);
    }
  };

  return (
    <div className="bg-background-surface border border-border rounded-panel p-5 shadow-xs flex flex-col gap-5">
      {/* 1. Header Card with Realtime Status */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-border/70">
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

      {/* 2. Process groups */}
      <div className="flex flex-col gap-4 divide-y divide-border/60">
        {/* Group 1: Host Core */}
        <div className="flex flex-col gap-2 pt-1 first:pt-0">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-blue-50 text-primary flex items-center justify-center">
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
            <div
              key={item.pid}
              className="p-3 bg-background-app/50 border border-border/50 rounded-control flex flex-col sm:flex-row sm:items-center justify-between gap-2"
            >
              <div className="flex flex-col gap-0.5">
                <span className="text-small font-medium text-content-primary">
                  {item.name}
                </span>
                <span className="text-caption text-content-muted">
                  PID: {item.pid} · Win32 Message Loop & Skia Vector Engine
                </span>
              </div>
              <div className="flex items-center gap-4 text-small">
                <div className="flex flex-col items-end">
                  <span className="font-semibold text-content-primary">
                    {item.workingSetMb.toFixed(1)} MB
                  </span>
                  <span className="text-caption text-content-secondary">
                    {t("ram_working_set")}
                  </span>
                </div>
                <div className="flex flex-col items-end w-14">
                  <span className="font-semibold text-content-primary">
                    {item.cpuPercent.toFixed(1)}%
                  </span>
                  <span className="text-caption text-content-secondary">
                    {t("cpu")}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Group 2: Dashboard UI */}
        <div className="flex flex-col gap-2 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-purple-50 text-purple-600 flex items-center justify-center">
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
            <div className="p-3 bg-background-app/30 border border-border/40 rounded-control text-caption text-content-muted">
              {t("dashboard_hibernating")}
            </div>
          ) : (
            dashboardItems.map((item) => (
              <div
                key={item.pid}
                className="p-3 bg-background-app/50 border border-border/50 rounded-control flex flex-col sm:flex-row sm:items-center justify-between gap-2"
              >
                <div className="flex flex-col gap-0.5">
                  <span className="text-small font-medium text-content-primary">
                    {item.name}
                  </span>
                  <span className="text-caption text-content-muted">
                    PID: {item.pid} · React 19 Dashboard UI
                  </span>
                </div>
                <div className="flex items-center gap-4 text-small">
                  <div className="flex flex-col items-end">
                    <span className="font-semibold text-content-primary">
                      {item.workingSetMb.toFixed(1)} MB
                    </span>
                    <span className="text-caption text-content-secondary">
                      {t("ram")}
                    </span>
                  </div>
                  <div className="flex flex-col items-end w-14">
                    <span className="font-semibold text-content-primary">
                      {item.cpuPercent.toFixed(1)}%
                    </span>
                    <span className="text-caption text-content-secondary">
                      {t("cpu")}
                    </span>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Group 3: Messenger Accounts */}
        <div className="flex flex-col gap-2.5 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-sky-50 text-sky-600 flex items-center justify-center">
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
            <div className="p-4 bg-background-app/30 border border-border/40 rounded-control text-center text-caption text-content-muted">
              {t("no_accounts_linked")}
            </div>
          ) : (
            messengerAccounts.map(([accountIndex, procList]) => {
              const isSuspended = procList.every((p) => p.isSuspended);
              const totalAccountRam = procList.reduce(
                (sum, p) => sum + p.workingSetMb,
                0,
              );
              const totalAccountCpu = procList.reduce(
                (sum, p) => sum + p.cpuPercent,
                0,
              );
              const displayName =
                procList[0]?.accountName ||
                `Messenger ${accountIndex.toString().padStart(2, "0")}`;
              const isActing = actingAccountIndex === accountIndex;

              return (
                <div
                  key={accountIndex}
                  className={`p-3.5 border rounded-panel flex flex-col gap-3 transition-colors ${
                    isSuspended
                      ? "bg-slate-50/60 border-slate-200"
                      : "bg-background-surface border-border shadow-xs"
                  }`}
                >
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5">
                    <div className="flex items-center gap-2.5">
                      <div
                        className={`w-9 h-9 rounded-xl flex items-center justify-center font-bold text-small shrink-0 ${
                          isSuspended
                            ? "bg-slate-200 text-slate-500"
                            : "bg-gradient-to-br from-blue-500 to-sky-400 text-white shadow-xs"
                        }`}
                      >
                        {accountIndex.toString().padStart(2, "0")}
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
                          {t("profile_dir")}: profiles/messenger_
                          {accountIndex.toString().padStart(2, "0")}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-3 self-end sm:self-auto">
                      <div className="flex items-center gap-3 text-small mr-2">
                        <div className="flex flex-col items-end">
                          <span
                            className={`font-semibold ${
                              isSuspended
                                ? "text-emerald-600 font-bold"
                                : "text-content-primary"
                            }`}
                          >
                            {isSuspended
                              ? "0 MB"
                              : `${totalAccountRam.toFixed(1)} MB`}
                          </span>
                          <span className="text-caption text-content-muted">
                            {isSuspended ? t("savings_tag") : t("total_ram")}
                          </span>
                        </div>
                        <div className="flex flex-col items-end w-12">
                          <span className="font-semibold text-content-primary">
                            {isSuspended
                              ? "0.0%"
                              : `${totalAccountCpu.toFixed(1)}%`}
                          </span>
                          <span className="text-caption text-content-muted">
                            {t("cpu")}
                          </span>
                        </div>
                      </div>
                      {isSuspended ? (
                        <button
                          onClick={() => handleWakeAccount(accountIndex)}
                          disabled={isActing}
                          className="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-hover text-white rounded-control text-small font-medium shadow-xs transition-colors cursor-pointer disabled:opacity-60"
                          title={t("wake_tooltip")}
                        >
                          <Play
                            className={`w-3.5 h-3.5 fill-current ${isActing ? "animate-spin" : ""}`}
                          />
                          <span>
                            {isActing ? t("wake_running") : t("wake_account")}
                          </span>
                        </button>
                      ) : (
                        <button
                          onClick={() => handleFreezeAccount(accountIndex)}
                          disabled={isActing}
                          className="flex items-center gap-1.5 px-3 py-1.5 bg-sky-50 hover:bg-sky-100 text-sky-700 border border-sky-200 rounded-control text-small font-medium transition-colors cursor-pointer disabled:opacity-60"
                          title={t("freeze_tooltip")}
                        >
                          <Snowflake
                            className={`w-3.5 h-3.5 ${isActing ? "animate-spin" : ""}`}
                          />
                          <span>
                            {isActing
                              ? t("freeze_running")
                              : t("freeze_account")}
                          </span>
                        </button>
                      )}
                    </div>
                  </div>
                  {!isSuspended && procList.length > 0 && (
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-2 border-t border-border/40">
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
                            {p.workingSetMb.toFixed(1)} MB ·{" "}
                            {p.cpuPercent.toFixed(1)}%
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>

        {/* Group 4: GPU & Utility */}
        <div className="flex flex-col gap-2 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-emerald-50 text-emerald-600 flex items-center justify-center">
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
              <div
                key={item.pid}
                className="p-2.5 bg-background-app/40 border border-border/40 rounded-control flex items-center justify-between text-caption"
              >
                <div className="flex flex-col">
                  <span className="font-medium text-content-primary">
                    {item.name}
                  </span>
                  <span className="text-content-muted">PID: {item.pid}</span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="font-semibold text-content-primary">
                    {item.workingSetMb.toFixed(1)} MB
                  </span>
                  <span className="text-content-secondary">
                    {item.cpuPercent.toFixed(1)}% {t("cpu")}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default ProcessDetailCard;
