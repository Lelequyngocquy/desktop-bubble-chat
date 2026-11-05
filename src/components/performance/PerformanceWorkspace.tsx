import React from "react";
import { Sparkles } from "lucide-react";
import { TabHeader } from "../common/TabHeader";
import {
  usePerformanceStore,
  usePerformanceAutoPoll,
} from "../../stores/performanceStore";
import { PerformanceSummaryCards } from "./PerformanceSummaryCards";
import { MemoryDistributionBar } from "./MemoryDistributionBar";
import { ProcessDetailCard } from "./ProcessDetailCard";

export const PerformanceWorkspace: React.FC = () => {
  // Tự động kích hoạt polling 1.5s khi mount và ngắt khi chuyển tab (0% CPU background)
  usePerformanceAutoPoll();

  const snapshot = usePerformanceStore((s) => s.snapshot);
  const isLoading = usePerformanceStore((s) => s.isLoading);
  const isTrimming = usePerformanceStore((s) => s.isTrimming);
  const lastTrimmedMb = usePerformanceStore((s) => s.lastTrimmedMb);
  const fetchSnapshot = usePerformanceStore((s) => s.fetchSnapshot);
  const trimMemory = usePerformanceStore((s) => s.trimMemory);

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      {/* 1. Header cố định */}
      <TabHeader
        eyebrow="HỆ THỐNG & TÀI NGUYÊN"
        title="Hiệu năng"
        description="Giám sát mức chiếm dụng RAM, CPU thời gian thực của từng tài khoản và toàn bộ hệ thống."
        rightSlot={
          <div className="flex items-center gap-2.5">
            {/* Huy hiệu nhịp tim Real-time */}
            <div className="flex items-center gap-2 px-3 py-1.5 bg-background-surface border border-border rounded-pill text-small shadow-xs">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
              <span className="text-content-primary font-medium text-caption">
                Giám sát Real-time (1.5s)
              </span>
            </div>

            {/* Nút hành động nhanh: Dọn RAM */}
            <button
              onClick={() => trimMemory()}
              disabled={isTrimming}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-hover text-white rounded-pill text-caption font-semibold shadow-xs transition-colors cursor-pointer disabled:opacity-60"
              title="Dọn dẹp và thu hồi bộ nhớ đệm Working Set về hệ thống"
            >
              <Sparkles className={`w-3.5 h-3.5 ${isTrimming ? "animate-spin" : ""}`} />
              <span>{isTrimming ? "Đang thu hồi..." : "Dọn dẹp RAM"}</span>
            </button>
          </div>
        }
      />

      {/* 2. Nội dung cuộn chính */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {/* 3 Thẻ KPI tổng quan */}
        <PerformanceSummaryCards snapshot={snapshot} isLoading={isLoading} />

        {/* Thanh phân bổ bộ nhớ */}
        <MemoryDistributionBar snapshot={snapshot} />

        {/* Chi tiết từng tiến trình & hành động */}
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
