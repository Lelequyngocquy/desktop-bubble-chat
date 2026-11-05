import React, { useState } from "react";
import {
  Layers,
  Sparkles,
  RefreshCw,
  Snowflake,
  Play,
  CheckCircle2,
  Cpu,
  Monitor,
  HardDrive,
  Info,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { ProcessMetricItem, SystemPerformanceSnapshot } from "../../types/performance";
import { IPC_COMMANDS } from "../../types/ipc";

interface ProcessDetailCardProps {
  snapshot: SystemPerformanceSnapshot | null;
  isLoading: boolean;
  isTrimming: boolean;
  lastTrimmedMb: number | null;
  onRefresh: () => void;
  onTrimMemory: () => Promise<number>;
}

export const ProcessDetailCard: React.FC<ProcessDetailCardProps> = ({
  snapshot,
  isLoading,
  isTrimming,
  lastTrimmedMb,
  onRefresh,
  onTrimMemory,
}) => {
  const [actingAccountIndex, setActingAccountIndex] = useState<number | null>(null);

  // Nhóm các items theo danh mục
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

  // Danh sách các accountIndex đã sắp xếp
  const messengerAccounts = Array.from(messengerItemsMap.entries()).sort(
    ([a], [b]) => a - b
  );

  const handleFreezeAccount = async (accountIndex: number) => {
    try {
      setActingAccountIndex(accountIndex);
      await invoke(IPC_COMMANDS.TOGGLE_BUBBLE, {
        accountIndex,
        visible: false,
      });
      // Đợi ngắn để tiến trình kịp terminate rồi refresh snapshot
      setTimeout(() => {
        onRefresh();
        setActingAccountIndex(null);
      }, 500);
    } catch (err) {
      console.error("[ProcessDetailCard] Lỗi khi đóng băng tài khoản:", err);
      setActingAccountIndex(null);
    }
  };

  const handleWakeAccount = async (accountIndex: number) => {
    try {
      setActingAccountIndex(accountIndex);
      await invoke(IPC_COMMANDS.OPEN_CHAT, { accountIndex });
      setTimeout(() => {
        onRefresh();
        setActingAccountIndex(null);
      }, 800);
    } catch (err) {
      console.error("[ProcessDetailCard] Lỗi khi kích hoạt tài khoản:", err);
      setActingAccountIndex(null);
    }
  };

  return (
    <div className="bg-background-surface border border-border rounded-panel p-5 shadow-xs flex flex-col gap-5">
      {/* 1. Header Card & Action Buttons */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-border/70">
        <div className="flex items-center gap-2">
          <Layers className="w-4 h-4 text-primary" />
          <h2 className="text-body-lg font-semibold text-content-primary m-0">
            Chi Tiết Tiến Trình & Tài Nguyên Hệ Thống
          </h2>
        </div>

        <div className="flex items-center gap-2">
          {/* Nút Dọn dẹp RAM (Trim Working Set) */}
          <button
            onClick={onTrimMemory}
            disabled={isTrimming}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-50 hover:bg-blue-100 text-primary border border-blue-200 rounded-control text-small font-medium transition-colors cursor-pointer disabled:opacity-60"
            title="Đẩy bộ nhớ đệm không dùng về Pagefile. Windows sẽ tự động cấp phát lại khi cần."
          >
            <Sparkles className={`w-3.5 h-3.5 ${isTrimming ? "animate-spin" : ""}`} />
            <span>{isTrimming ? "Đang dọn dẹp..." : "Dọn dẹp RAM"}</span>
          </button>

          {/* Nút Làm mới thủ công */}
          <button
            onClick={onRefresh}
            disabled={isLoading}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-background-surface hover:bg-white border border-border rounded-control text-small text-content-secondary hover:text-content-primary transition-colors cursor-pointer"
            title="Làm mới số liệu đo lường ngay lập tức"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? "animate-spin text-primary" : ""}`} />
            <span>{isLoading ? "Đang quét..." : "Làm mới"}</span>
          </button>
        </div>
      </div>

      {/* Thông báo kết quả Trim RAM nếu có */}
      {lastTrimmedMb !== null && lastTrimmedMb > 0 && (
        <div className="p-3 bg-emerald-50 border border-emerald-200 rounded-control text-emerald-800 text-small flex items-center gap-2">
          <CheckCircle2 className="w-4 h-4 text-emerald-600 shrink-0" />
          <span>
            Đã thu hồi thành công <strong>{lastTrimmedMb.toFixed(1)} MB</strong> bộ nhớ đệm về hệ điều hành Windows!
          </span>
        </div>
      )}

      {/* 2. Danh sách các nhóm tiến trình */}
      <div className="flex flex-col gap-4 divide-y divide-border/60">
        {/* Nhóm 1: Hệ thống cốt lõi (Rust Host Core) */}
        <div className="flex flex-col gap-2 pt-1 first:pt-0">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-blue-50 text-primary flex items-center justify-center">
                <Cpu className="w-3.5 h-3.5" />
              </div>
              <span className="text-small font-semibold text-content-primary">
                1. Hệ Thống Cốt Lõi (Rust Host & Native Bubbles)
              </span>
            </div>
            <span className="text-caption text-emerald-600 font-medium flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
              Hoạt động tối ưu
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
                    RAM Working Set
                  </span>
                </div>
                <div className="flex flex-col items-end w-14">
                  <span className="font-semibold text-content-primary">
                    {item.cpuPercent.toFixed(1)}%
                  </span>
                  <span className="text-caption text-content-secondary">
                    CPU
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Nhóm 2: Bảng điều khiển (Dashboard UI) */}
        <div className="flex flex-col gap-2 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-purple-50 text-purple-600 flex items-center justify-center">
                <Monitor className="w-3.5 h-3.5" />
              </div>
              <span className="text-small font-semibold text-content-primary">
                2. Bảng Điều Khiển (Dashboard WebView2 Main Window)
              </span>
            </div>
            <span className="text-caption text-content-muted flex items-center gap-1">
              <Info className="w-3.5 h-3.5" />
              Tự động ngủ đông 0 MB khi đóng về khay
            </span>
          </div>

          {dashboardItems.length === 0 ? (
            <div className="p-3 bg-background-app/30 border border-border/40 rounded-control text-caption text-content-muted">
              Cửa sổ Dashboard đang ở chế độ ngủ đông (0 MB UI RAM)
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
                      RAM
                    </span>
                  </div>
                  <div className="flex flex-col items-end w-14">
                    <span className="font-semibold text-content-primary">
                      {item.cpuPercent.toFixed(1)}%
                    </span>
                    <span className="text-caption text-content-secondary">
                      CPU
                    </span>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Nhóm 3: Các tài khoản Messenger (01..05) */}
        <div className="flex flex-col gap-2.5 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-sky-50 text-sky-600 flex items-center justify-center">
                <Snowflake className="w-3.5 h-3.5" />
              </div>
              <span className="text-small font-semibold text-content-primary">
                3. Các Tài Khoản Messenger Độc Lập
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              Mỗi tài khoản là 1 phiên WebView2 cô lập hoàn toàn
            </span>
          </div>

          {messengerAccounts.length === 0 ? (
            <div className="p-4 bg-background-app/30 border border-border/40 rounded-control text-center text-caption text-content-muted">
              Chưa có tài khoản Messenger nào được liên kết
            </div>
          ) : (
            messengerAccounts.map(([accountIndex, procList]) => {
              const isSuspended = procList.every((p) => p.isSuspended);
              const totalAccountRam = procList.reduce((sum, p) => sum + p.workingSetMb, 0);
              const totalAccountCpu = procList.reduce((sum, p) => sum + p.cpuPercent, 0);
              const displayName = procList[0]?.accountName || `Messenger ${accountIndex.toString().padStart(2, "0")}`;
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
                            <span className="px-2 py-0.5 rounded-pill text-[11px] font-semibold bg-sky-50 text-sky-700 border border-sky-200">
                              ❄️ Đã đóng băng
                            </span>
                          ) : (
                            <span className="px-2 py-0.5 rounded-pill text-[11px] font-semibold bg-emerald-50 text-emerald-700 border border-emerald-200">
                              🟢 Đang hoạt động
                            </span>
                          )}
                        </div>
                        <span className="text-caption text-content-muted">
                          Thư mục hồ sơ: profiles/messenger_{accountIndex.toString().padStart(2, "0")}
                        </span>
                      </div>
                    </div>

                    {/* Chỉ số & Nút hành động */}
                    <div className="flex items-center gap-3 self-end sm:self-auto">
                      <div className="flex items-center gap-3 text-small mr-2">
                        <div className="flex flex-col items-end">
                          <span
                            className={`font-semibold ${
                              isSuspended ? "text-emerald-600 font-bold" : "text-content-primary"
                            }`}
                          >
                            {isSuspended ? "0 MB" : `${totalAccountRam.toFixed(1)} MB`}
                          </span>
                          <span className="text-caption text-content-muted">
                            {isSuspended ? "Tiết kiệm ~700MB" : "Tổng RAM"}
                          </span>
                        </div>
                        <div className="flex flex-col items-end w-12">
                          <span className="font-semibold text-content-primary">
                            {isSuspended ? "0.0%" : `${totalAccountCpu.toFixed(1)}%`}
                          </span>
                          <span className="text-caption text-content-muted">CPU</span>
                        </div>
                      </div>

                      {/* Nút Đóng băng / Kích hoạt */}
                      {isSuspended ? (
                        <button
                          onClick={() => handleWakeAccount(accountIndex)}
                          disabled={isActing}
                          className="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-hover text-white rounded-control text-small font-medium shadow-xs transition-colors cursor-pointer disabled:opacity-60"
                          title="Khởi động lại bong bóng và cửa sổ chat cho tài khoản này"
                        >
                          <Play className={`w-3.5 h-3.5 fill-current ${isActing ? "animate-spin" : ""}`} />
                          <span>{isActing ? "Đang mở..." : "Kích hoạt"}</span>
                        </button>
                      ) : (
                        <button
                          onClick={() => handleFreezeAccount(accountIndex)}
                          disabled={isActing}
                          className="flex items-center gap-1.5 px-3 py-1.5 bg-sky-50 hover:bg-sky-100 text-sky-700 border border-sky-200 rounded-control text-small font-medium transition-colors cursor-pointer disabled:opacity-60"
                          title="Hủy tiến trình WebView2 để giải phóng ngay ~700 MB RAM về máy tính"
                        >
                          <Snowflake className={`w-3.5 h-3.5 ${isActing ? "animate-spin" : ""}`} />
                          <span>{isActing ? "Đang xử lý..." : "Đóng băng ngay"}</span>
                        </button>
                      )}
                    </div>
                  </div>

                  {/* Chi tiết từng tiến trình con nếu tài khoản đang chạy */}
                  {!isSuspended && procList.length > 0 && (
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-2 border-t border-border/40">
                      {procList.map((p) => (
                        <div
                          key={p.pid}
                          className="px-2.5 py-1.5 bg-background-app/50 rounded-control text-caption flex items-center justify-between"
                        >
                          <span className="text-content-secondary truncate max-w-[180px]">
                            {p.name} (PID: {p.pid})
                          </span>
                          <span className="font-semibold text-content-primary">
                            {p.workingSetMb.toFixed(1)} MB · {p.cpuPercent.toFixed(1)}%
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

        {/* Nhóm 4: GPU Acceleration & Utility Brokers */}
        <div className="flex flex-col gap-2 pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-emerald-50 text-emerald-600 flex items-center justify-center">
                <HardDrive className="w-3.5 h-3.5" />
              </div>
              <span className="text-small font-semibold text-content-primary">
                4. Dịch Vụ Nền & Tăng Tốc Phần Cứng (GPU & Utility Brokers)
              </span>
            </div>
            <span className="text-caption text-content-muted">
              Tiến trình chia sẻ của WebView2 Runtime
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
                    {item.cpuPercent.toFixed(1)}% CPU
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
