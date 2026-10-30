import React, { useState } from "react";
import {
  Power,
  Keyboard,
  CircleDot,
  Trash2,
  Check,
  RefreshCw,
  HardDrive,
} from "lucide-react";
import {
  useSettingsStore,
  BubbleSize,
  BUBBLE_SIZE_PIXELS,
} from "../../stores/settingsStore";
import { systemService } from "../../services/systemService";
import { messengerService } from "../../services/messenger";
import { notificationService } from "../../services/notificationService";
import { SwitchButton, TabHeader, CommonDialog } from "../common";

export const SettingsWorkspace: React.FC = () => {
  const isAlwaysOnTop = useSettingsStore((s) => s.isAlwaysOnTop);
  const toggleAlwaysOnTop = useSettingsStore((s) => s.toggleAlwaysOnTop);
  const isNotificationsEnabled = useSettingsStore(
    (s) => s.isNotificationsEnabled,
  );
  const toggleNotifications = useSettingsStore((s) => s.toggleNotifications);
  const isAutoStart = useSettingsStore((s) => s.isAutoStart);
  const toggleAutoStart = useSettingsStore((s) => s.toggleAutoStart);
  const isGlobalShortcutEnabled = useSettingsStore(
    (s) => s.isGlobalShortcutEnabled,
  );
  const toggleGlobalShortcut = useSettingsStore((s) => s.toggleGlobalShortcut);
  const bubbleSize = useSettingsStore((s) => s.bubbleSize);
  const setBubbleSize = useSettingsStore((s) => s.setBubbleSize);

  const [clearStatus, setClearStatus] = useState<string | null>(null);
  const [isConfirmClearCacheOpen, setIsConfirmClearCacheOpen] = useState(false);

  const handleToggleAutoStart = async () => {
    const nextVal = !isAutoStart;
    toggleAutoStart();
    await systemService.toggleAutostart(nextVal);
  };

  const handleToggleAlwaysOnTop = async () => {
    const nextVal = !isAlwaysOnTop;
    toggleAlwaysOnTop();
    await systemService.setAlwaysOnTop(nextVal);
  };

  const handleToggleNotifications = async () => {
    const nextVal = !isNotificationsEnabled;
    toggleNotifications();
    if (nextVal) {
      await notificationService.requestPermission();
    }
  };

  const handleSelectBubbleSize = async (size: BubbleSize) => {
    setBubbleSize(size);
    await messengerService.setBubbleConfig(BUBBLE_SIZE_PIXELS[size]);
  };

  const handleClearCache = async () => {
    try {
      setClearStatus("clearing");
      await messengerService.clearStorage(1);
      setClearStatus("success");
      setTimeout(() => setClearStatus(null), 3000);
    } catch {
      setClearStatus("error");
      setTimeout(() => setClearStatus(null), 3000);
    }
  };

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-[#F4F7FC] overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="TÙY CHỈNH HỆ THỐNG"
        title="Cài đặt"
        description="Tùy biến cấu hình hệ thống, kích thước bong bóng nổi, phím tắt và bộ nhớ đệm."
        rightSlot={
          <div className="flex items-center gap-1.5 px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-full text-[12px] shadow-xs">
            <span className="w-2 h-2 rounded-full bg-[#10B981]" />
            <span className="text-[#202733] font-medium">Phiên bản v0.1.0</span>
          </div>
        }
      />

      {/* 2. Nội dung cuộn mượt mà luôn bung hết chiều rộng */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {/* Section 1: Hệ thống & Khởi động */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
          <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
            <Power className="w-4 h-4 text-[#0866FF]" />
            <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
              Hệ thống & Khởi động
            </h2>
          </div>

          <div className="flex flex-col divide-y divide-[#E2E7EF]">
            <div className="py-3 flex items-center justify-between gap-4">
              <div className="flex flex-col gap-0.5">
                <span className="text-[13px] font-medium text-[#1E293B]">
                  Khởi động cùng Windows
                </span>
                <span className="text-[11px] text-[#64748B]">
                  Tự động bật DesktopBubbleChat khi bạn đăng nhập vào máy tính
                </span>
              </div>
              <SwitchButton
                checked={isAutoStart}
                onChange={handleToggleAutoStart}
              />
            </div>

            <div className="py-3 flex items-center justify-between gap-4">
              <div className="flex flex-col gap-0.5">
                <span className="text-[13px] font-medium text-[#1E293B]">
                  Bong bóng nổi trên cùng (Tắt vẫn có thể mở khung trò chuyện
                  nếu di chuột đúng vị trí)
                </span>
                <span className="text-[11px] text-[#64748B]">
                  Giữ các bong bóng chat luôn hiển thị trên các ứng dụng khác.
                  Khi tắt, các ứng dụng khác có thể che phủ bong bóng.
                </span>
              </div>
              <SwitchButton
                checked={isAlwaysOnTop}
                onChange={handleToggleAlwaysOnTop}
              />
            </div>

            <div className="py-3 flex items-center justify-between gap-4">
              <div className="flex flex-col gap-0.5">
                <span className="text-[13px] font-medium text-[#1E293B]">
                  Thông báo desktop & Âm thanh
                </span>
                <span className="text-[11px] text-[#64748B]">
                  Phát âm thanh bong bóng nhẹ và hiển thị thông báo toast khi có
                  tin nhắn mới
                </span>
              </div>
              <SwitchButton
                checked={isNotificationsEnabled}
                onChange={handleToggleNotifications}
              />
            </div>
          </div>
        </div>

        {/* Section 2: Tùy chỉnh Bong bóng nổi */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
          <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
            <CircleDot className="w-4 h-4 text-[#0866FF]" />
            <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
              Tùy biến Bong bóng nổi
            </h2>
          </div>

          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <span className="text-[13px] font-medium text-[#1E293B]">
                Kích thước hiển thị
              </span>
              <div className="grid grid-cols-3 gap-3 max-w-[480px]">
                {(
                  [
                    {
                      id: "sm",
                      label: "Nhỏ (48px)",
                      desc: "Gọn gàng, ít choán màn hình",
                    },
                    {
                      id: "md",
                      label: "Vừa (56px)",
                      desc: "Kích thước chuẩn khuyến nghị",
                    },
                    {
                      id: "lg",
                      label: "Lớn (64px)",
                      desc: "Dễ click, avatar to rõ",
                    },
                  ] as const
                ).map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => handleSelectBubbleSize(item.id)}
                    className={`p-3 rounded-xl border flex flex-col items-center justify-center text-center gap-1 transition-all cursor-pointer ${
                      bubbleSize === item.id
                        ? "border-[#0866FF] bg-[#EAF2FF] text-[#0759DB] font-semibold shadow-xs"
                        : "border-[#E2E7EF] bg-white text-[#475569] hover:border-[#CBD5E1]"
                    }`}
                  >
                    <span className="text-[13px]">{item.label}</span>
                    <span className="text-[10px] text-[#64748B] leading-tight">
                      {item.desc}
                    </span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Section 3: Phím tắt toàn cục */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
          <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
            <Keyboard className="w-4 h-4 text-[#0866FF]" />
            <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
              Phím tắt toàn cục
            </h2>
          </div>

          <div className="flex items-center justify-between gap-4 py-1">
            <div className="flex flex-col gap-0.5">
              <div className="flex items-center gap-2">
                <span className="text-[13px] font-medium text-[#1E293B]">
                  Bật/Tắt nhanh bong bóng chat
                </span>
                <span className="px-2 py-0.5 bg-[#F1F5F9] border border-[#CBD5E1] rounded text-[11px] font-mono text-[#334155] font-semibold">
                  Ctrl + Shift + M
                </span>
              </div>
              <span className="text-[11px] text-[#64748B]">
                Bấm tổ hợp phím này ở bất cứ đâu trong Windows để ẩn hoặc hiện
                toàn bộ bong bóng nổi
              </span>
            </div>
            <SwitchButton
              checked={isGlobalShortcutEnabled}
              onChange={toggleGlobalShortcut}
            />
          </div>
        </div>

        {/* Section 4: Dữ liệu & Bộ nhớ đệm */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
          <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
            <HardDrive className="w-4 h-4 text-[#0866FF]" />
            <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
              Dữ liệu & Bộ nhớ đệm
            </h2>
          </div>

          <div className="flex items-center justify-between gap-4 py-1">
            <div className="flex flex-col gap-0.5">
              <span className="text-[13px] font-medium text-[#1E293B]">
                Xóa bộ nhớ đệm WebView2
              </span>
              <span className="text-[11px] text-[#64748B]">
                Giải phóng dung lượng ổ đĩa và làm mới hoàn toàn session nếu gặp
                lỗi hiển thị
              </span>
            </div>
            <button
              type="button"
              onClick={() => setIsConfirmClearCacheOpen(true)}
              disabled={clearStatus === "clearing"}
              className="px-3.5 py-1.5 border border-[#E2E7EF] rounded-lg text-[12px] font-medium text-[#DC2626] hover:bg-[#FEECEE] flex items-center gap-1.5 transition-colors cursor-pointer disabled:opacity-50"
            >
              {clearStatus === "clearing" ? (
                <>
                  <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                  <span>Đang dọn dẹp...</span>
                </>
              ) : clearStatus === "success" ? (
                <>
                  <Check className="w-3.5 h-3.5 text-green-600" />
                  <span className="text-green-600">Đã xóa thành công!</span>
                </>
              ) : (
                <>
                  <Trash2 className="w-3.5 h-3.5" />
                  <span>Xóa bộ nhớ đệm</span>
                </>
              )}
            </button>
          </div>

          <div className="pt-2 border-t border-[#E2E7EF]/50 flex items-center justify-between text-[11px] text-[#94A3B8]">
            <span>DesktopBubbleChat v0.1.0 (Windows 11 / WebView2)</span>
            <span>Build Clean Architecture</span>
          </div>
        </div>
      </main>

      {/* Dialog xác nhận xóa bộ nhớ đệm */}
      <CommonDialog
        isOpen={isConfirmClearCacheOpen}
        onClose={() => setIsConfirmClearCacheOpen(false)}
        eyebrow="XÁC NHẬN DỌN DẸP"
        titleIcon={<Trash2 className="w-5 h-5 text-[#DC2626]" />}
        title="Xóa bộ nhớ đệm WebView2"
        content="Bạn có chắc chắn muốn xóa bộ nhớ đệm WebView2? Thao tác này sẽ giải phóng dung lượng ổ đĩa và làm mới hoàn toàn session dữ liệu của trình duyệt nhúng."
        widget={
          <div className="p-3 bg-[#FFF5F5] rounded-lg border border-[#FEE2E2] text-[12px] text-[#991B1B] leading-relaxed">
            Lưu ý: Sau khi xóa bộ nhớ đệm, các phiên đăng nhập webview có thể
            cần tải lại tài nguyên trong lần mở kế tiếp.
          </div>
        }
        backButton={{
          label: "Hủy bỏ",
          onClick: () => setIsConfirmClearCacheOpen(false),
        }}
        rightButton={{
          label: "Xác nhận xóa",
          variant: "danger",
          loading: clearStatus === "clearing",
          onClick: async () => {
            setIsConfirmClearCacheOpen(false);
            await handleClearCache();
          },
        }}
      />
    </div>
  );
};
