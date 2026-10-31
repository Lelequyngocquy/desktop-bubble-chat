import React, { useState } from "react";
import { MessageSquare, Power, HelpCircle, ShieldAlert } from "lucide-react";
import { CommonDialog } from "../../common/CommonDialog";

export interface CloseAppDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onHideToTray: () => void;
  onExitApp: () => void;
}

export const CloseAppDialog: React.FC<CloseAppDialogProps> = ({
  isOpen,
  onClose,
  onHideToTray,
  onExitApp,
}) => {
  const [rememberChoice, setRememberChoice] = useState(false);

  const handleConfirmHide = () => {
    if (rememberChoice) {
      localStorage.setItem("dbc_close_window_action", "hide_to_tray");
    }
    onHideToTray();
  };

  const handleConfirmExit = () => {
    if (rememberChoice) {
      localStorage.setItem("dbc_close_window_action", "exit_app");
    }
    onExitApp();
  };

  return (
    <CommonDialog
      isOpen={isOpen}
      onClose={onClose}
      eyebrow="HỆ THỐNG"
      title="Tùy chọn đóng ứng dụng"
      titleIcon={<HelpCircle className="w-5 h-5 text-[#0866FF]" />}
      size="md"
      footerLayout="split"
      backButton={{
        label: "Hủy bỏ",
        variant: "outline",
        onClick: onClose,
      }}
      rightButton={{
        label: "Đóng cửa sổ",
        variant: "primary",
        onClick: handleConfirmHide,
      }}
      content={
        <div className="space-y-4 text-[13px] text-[#4A5568]">
          <p className="m-0 leading-relaxed">
            Bạn có muốn tiếp tục chạy ứng dụng dưới khay hệ thống (System Tray) để nhận thông báo và duy trì bong bóng nổi không?
          </p>

          {/* Card giải thích Chạy ngầm */}
          <div className="flex items-start gap-3 p-3 bg-[#F0F4FF] border border-[#D0E0FF] rounded-[8px] text-[#1E3A8A]">
            <MessageSquare className="w-4 h-4 mt-0.5 shrink-0 text-[#0866FF]" />
            <div className="text-[12px] leading-relaxed">
              <span className="font-semibold">Chạy trong khay hệ thống:</span> Cửa sổ giao diện chính sẽ đóng lại, ứng dụng tiếp tục chạy ngầm ở góc màn hình. Bong bóng chat nổi và chuông thông báo vẫn hoạt động bình thường.
            </div>
          </div>

          {/* Tùy chọn Thoát hoàn toàn */}
          <div className="pt-2 border-t border-[#E2E8F0]">
            <div className="flex items-center justify-between p-2.5 rounded-[8px] border border-red-100 bg-red-50/60 hover:bg-red-50 transition-colors">
              <div className="flex items-center gap-2.5">
                <ShieldAlert className="w-4 h-4 text-red-500 shrink-0" />
                <span className="text-[12px] text-red-900 font-medium">
                  Tắt hoàn toàn ứng dụng và bong bóng
                </span>
              </div>
              <button
                type="button"
                onClick={handleConfirmExit}
                className="inline-flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold text-red-700 bg-white border border-red-300 rounded-[6px] hover:bg-red-600 hover:text-white hover:border-red-600 transition-colors shadow-2xs cursor-pointer"
              >
                <Power className="w-3 h-3" />
                Thoát hoàn toàn
              </button>
            </div>
          </div>

          {/* Checkbox Ghi nhớ lựa chọn */}
          <div className="pt-1 flex items-center gap-2">
            <input
              type="checkbox"
              id="rememberCloseChoice"
              checked={rememberChoice}
              onChange={(e) => setRememberChoice(e.target.checked)}
              className="w-4 h-4 rounded border-[#CBD5E1] text-[#0866FF] focus:ring-[#0866FF] cursor-pointer"
            />
            <label
              htmlFor="rememberCloseChoice"
              className="text-[12px] text-[#64748B] cursor-pointer select-none"
            >
              Ghi nhớ lựa chọn này (tự động chạy ngầm ở các lần sau)
            </label>
          </div>
        </div>
      }
    />
  );
};

export default CloseAppDialog;
