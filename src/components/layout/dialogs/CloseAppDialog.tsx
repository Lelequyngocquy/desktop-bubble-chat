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
      titleIcon={<HelpCircle className="w-5 h-5 text-primary" />}
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
        <div className="space-y-4 text-body text-content-secondary">
          <p className="m-0 leading-relaxed">
            Bạn có muốn tiếp tục chạy ứng dụng dưới khay hệ thống (System Tray)
            để nhận thông báo và duy trì bong bóng nổi không?
          </p>

          {/* Card giải thích Chạy ngầm */}
          <div className="flex items-start gap-3 p-3 bg-primary-subtle border border-primary/20 rounded-panel text-content-primary">
            <MessageSquare className="w-4 h-4 mt-0.5 shrink-0 text-primary" />
            <div className="text-small leading-relaxed">
              <span className="font-semibold">Chạy trong khay hệ thống:</span>{" "}
              Cửa sổ giao diện chính sẽ đóng lại, ứng dụng tiếp tục chạy ngầm ở
              góc màn hình. Bong bóng chat nổi và chuông thông báo vẫn hoạt động
              bình thường. Điều này sẽ giúp tiết kiệm tài nguyên hệ thống và
              tránh làm gián đoạn công việc của bạn.
            </div>
          </div>

          {/* Tùy chọn Thoát hoàn toàn */}
          <div className="pt-2 border-t border-border">
            <div className="flex items-center justify-between p-2.5 rounded-panel border border-status-errorBorder bg-status-errorBg/60 hover:bg-status-errorBg transition-colors">
              <div className="flex items-center gap-2.5">
                <ShieldAlert className="w-4 h-4 text-status-error shrink-0" />
                <span className="text-small text-status-error font-medium">
                  Tắt hoàn toàn ứng dụng và bong bóng
                </span>
              </div>
              <button
                type="button"
                onClick={handleConfirmExit}
                className="inline-flex items-center gap-1.5 px-2.5 py-1 text-caption font-semibold text-status-error bg-background-surface border border-status-errorBorder rounded-control hover:bg-status-error hover:text-content-inverse hover:border-status-error transition-colors shadow-2xs cursor-pointer"
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
              className="w-4 h-4 rounded border-border text-primary focus:ring-primary cursor-pointer"
            />
            <label
              htmlFor="rememberCloseChoice"
              className="text-small text-content-secondary cursor-pointer select-none"
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
