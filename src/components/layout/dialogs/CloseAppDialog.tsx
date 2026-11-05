import React, { useState } from "react";
import { MessageSquare, Power, HelpCircle, ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation("common");
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
      eyebrow={t("close_dialog.eyebrow")}
      title={t("close_dialog.title")}
      titleIcon={<HelpCircle className="w-5 h-5 text-primary" />}
      size="md"
      footerLayout="split"
      backButton={{
        label: t("close_dialog.cancel"),
        variant: "outline",
        onClick: onClose,
      }}
      rightButton={{
        label: t("close_dialog.hide_to_tray"),
        variant: "primary",
        onClick: handleConfirmHide,
      }}
      content={
        <div className="space-y-4 text-body text-content-secondary">
          <p className="m-0 leading-relaxed">
            {t("close_dialog.desc")}
          </p>
          <div className="flex items-start gap-3 p-3 bg-primary-subtle border border-primary/20 rounded-panel text-content-primary">
            <MessageSquare className="w-4 h-4 mt-0.5 shrink-0 text-primary" />
            <div className="text-small leading-relaxed">
              <span className="font-semibold">{t("close_dialog.tray_card_label")}</span>{" "}
              {t("close_dialog.tray_card_desc")}
            </div>
          </div>
          <div className="pt-2 border-t border-border">
            <div className="flex items-center justify-between p-2.5 rounded-panel border border-status-errorBorder bg-status-errorBg/60 hover:bg-status-errorBg transition-colors">
              <div className="flex items-center gap-2.5">
                <ShieldAlert className="w-4 h-4 text-status-error shrink-0" />
                <span className="text-small text-status-error font-medium">
                  {t("close_dialog.exit_warning")}
                </span>
              </div>
              <button
                type="button"
                onClick={handleConfirmExit}
                className="inline-flex items-center gap-1.5 px-2.5 py-1 text-caption font-semibold text-status-error bg-background-surface border border-status-errorBorder rounded-control hover:bg-status-error hover:text-content-inverse hover:border-status-error transition-colors shadow-2xs cursor-pointer"
              >
                <Power className="w-3 h-3" />
                {t("close_dialog.exit_btn")}
              </button>
            </div>
          </div>
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
              {t("close_dialog.remember_choice")}
            </label>
          </div>
        </div>
      }
    />
  );
};

export default CloseAppDialog;