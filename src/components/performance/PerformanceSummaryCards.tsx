import React from "react";
import { Cpu, Activity, Snowflake } from "lucide-react";
import { SystemPerformanceSnapshot } from "../../types/performance";

interface PerformanceSummaryCardsProps {
  snapshot: SystemPerformanceSnapshot | null;
  isLoading: boolean;
}

export const PerformanceSummaryCards: React.FC<PerformanceSummaryCardsProps> = ({
  snapshot,
  isLoading,
}) => {
  const totalRamMb = snapshot?.totalRamMb ?? 0;
  const totalCpuPercent = snapshot?.totalCpuPercent ?? 0;
  const activeCount = snapshot?.activeAccountsCount ?? 0;
  const suspendedCount = snapshot?.suspendedAccountsCount ?? 0;
  const estimatedSavedMb = snapshot?.estimatedSavedRamMb ?? 0;
  const cpuCores = snapshot?.cpuCores ?? 4;

  const formattedRam =
    totalRamMb >= 1024
      ? `${(totalRamMb / 1024).toFixed(2)} GB`
      : `${Math.round(totalRamMb)} MB`;

  const ramSecondaryText =
    totalRamMb >= 1024 ? `~${Math.round(totalRamMb)} MB` : undefined;

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {/* Thẻ 1: Tổng RAM Chiếm Dụng */}
      <div className="bg-background-surface border border-border rounded-panel p-5 shadow-xs flex flex-col justify-between gap-3">
        <div className="flex items-center justify-between">
          <span className="text-caption font-semibold text-content-secondary uppercase tracking-wider">
            Tổng RAM Chiếm Dụng
          </span>
          <div className="w-8 h-8 rounded-lg bg-blue-50 text-primary flex items-center justify-center">
            <Cpu className="w-4 h-4" />
          </div>
        </div>

        <div className="flex flex-col gap-0.5">
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-content-primary tracking-tight">
              {isLoading && !snapshot ? "..." : formattedRam}
            </span>
            {ramSecondaryText && (
              <span className="text-small font-medium text-content-muted">
                ({ramSecondaryText})
              </span>
            )}
          </div>
          <span className="text-caption text-content-secondary">
            Bao gồm Rust Core, Dashboard & các tài khoản
          </span>
        </div>

        <div className="w-full h-1.5 bg-slate-100 rounded-pill overflow-hidden mt-1">
          <div
            className="h-full bg-primary rounded-pill transition-all duration-500"
            style={{
              width: `${Math.min(100, Math.max(8, (totalRamMb / 2048) * 100))}%`,
            }}
          />
        </div>
      </div>

      {/* Thẻ 2: Mức Sử Dụng CPU */}
      <div className="bg-background-surface border border-border rounded-panel p-5 shadow-xs flex flex-col justify-between gap-3">
        <div className="flex items-center justify-between">
          <span className="text-caption font-semibold text-content-secondary uppercase tracking-wider">
            Mức Sử Dụng CPU
          </span>
          <div className="w-8 h-8 rounded-lg bg-emerald-50 text-emerald-600 flex items-center justify-center">
            <Activity className="w-4 h-4" />
          </div>
        </div>

        <div className="flex flex-col gap-0.5">
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-content-primary tracking-tight">
              {isLoading && !snapshot ? "..." : `${totalCpuPercent.toFixed(1)}%`}
            </span>
            <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-emerald-50 text-emerald-700 border border-emerald-200">
              {totalCpuPercent < 5 ? "Rất nhẹ" : totalCpuPercent < 20 ? "Bình thường" : "Đang tải"}
            </span>
          </div>
          <span className="text-caption text-content-secondary">
            {cpuCores} luồng xử lý · Tối ưu với Rust Host & Win32 API
          </span>
        </div>

        <div className="w-full h-1.5 bg-slate-100 rounded-pill overflow-hidden mt-1">
          <div
            className="h-full bg-emerald-500 rounded-pill transition-all duration-500"
            style={{
              width: `${Math.min(100, Math.max(6, totalCpuPercent * 2))}%`,
            }}
          />
        </div>
      </div>

      {/* Thẻ 3: Trạng Thái Tài Khoản & Tiết Kiệm RAM */}
      <div className="bg-background-surface border border-border rounded-panel p-5 shadow-xs flex flex-col justify-between gap-3">
        <div className="flex items-center justify-between">
          <span className="text-caption font-semibold text-content-secondary uppercase tracking-wider">
            Tài Khoản & Tiết Kiệm
          </span>
          <div className="w-8 h-8 rounded-lg bg-sky-50 text-sky-600 flex items-center justify-center">
            <Snowflake className="w-4 h-4" />
          </div>
        </div>

        <div className="flex flex-col gap-0.5">
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-content-primary tracking-tight">
              {activeCount} chạy / {suspendedCount} đóng băng
            </span>
          </div>
          <div className="flex items-center gap-1.5 mt-0.5">
            <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-sky-50 text-sky-700 border border-sky-200 flex items-center gap-1">
              <span>❄️ Đã tiết kiệm ~{Math.round(estimatedSavedMb)} MB RAM</span>
            </span>
          </div>
        </div>

        <div className="text-caption text-content-muted mt-1">
          Tài khoản đóng băng đạt chuẩn 0 MB RAM & 0% CPU
        </div>
      </div>
    </div>
  );
};

export default PerformanceSummaryCards;
