import React, { useState } from "react";
import { MessageSquare, Power, HelpCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { CommonDialog } from "../../common/CommonDialog";
import { useSettingsStore } from "../../../stores/settingsStore";

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
  const [selectedAction, setSelectedAction] = useState<"hide_to_tray" | "exit_app">("hide_to_tray");
  const [rememberChoice, setRememberChoice] = useState(false);
  const handleConfirm = () => {
    if (rememberChoice) {
      useSettingsStore.getState().setCloseWindowAction(selectedAction);
    }
    if (selectedAction === "hide_to_tray") {
      onHideToTray();
    } else {
      onExitApp();
    }
  };
  const handleDirectHide = () => {
    if (rememberChoice) {
      useSettingsStore.getState().setCloseWindowAction("hide_to_tray");
    }
    onHideToTray();
  };
  const handleDirectExit = () => {
    if (rememberChoice) {
      useSettingsStore.getState().setCloseWindowAction("exit_app");
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
        label:
          selectedAction === "hide_to_tray"
            ? t("close_dialog.action_hide")
            : t("close_dialog.action_exit"),
        variant: selectedAction === "hide_to_tray" ? "primary" : "danger",
        onClick: handleConfirm,
      }}
      content={
        <div className="space-y-3.5 text-body text-content-secondary">
          <p className="m-0 text-small text-content-secondary leading-relaxed">
            {t("close_dialog.desc")}
          </p>
          <div className="grid grid-cols-1 gap-2.5">
            <div
              onClick={() => setSelectedAction("hide_to_tray")}
              onDoubleClick={handleDirectHide}
              className={`p-3.5 rounded-panel border transition-all cursor-pointer select-none flex items-start gap-3 ${
                selectedAction === "hide_to_tray"
                  ? "bg-primary-subtle border-primary ring-1 ring-primary"
                  : "bg-background-surface border-border hover:border-border hover:bg-background-subtle"
              }`}
            >
              <div
                className={`w-8 h-8 rounded-control flex items-center justify-center shrink-0 ${
                  selectedAction === "hide_to_tray"
                    ? "bg-primary text-content-inverse"
                    : "bg-background-subtle text-content-secondary"
                }`}
              >
                <MessageSquare className="w-4 h-4" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between gap-2">
                  <span className="text-small font-semibold text-content-primary">
                    {t("close_dialog.hide_title")}
                  </span>
                  <span className="text-[11px] font-medium px-2 py-0.5 rounded-pill bg-primary/10 text-primary shrink-0">
                    {t("close_dialog.hide_badge")}
                  </span>
                </div>
                <p className="mt-1 mb-0 text-caption text-content-secondary leading-relaxed">
                  {t("close_dialog.hide_desc")}
                </p>
              </div>
            </div>
            <div
              onClick={() => setSelectedAction("exit_app")}
              onDoubleClick={handleDirectExit}
              className={`p-3.5 rounded-panel border transition-all cursor-pointer select-none flex items-start gap-3 ${
                selectedAction === "exit_app"
                  ? "bg-status-errorBg border-status-error ring-1 ring-status-error"
                  : "bg-background-surface border-border hover:border-border hover:bg-background-subtle"
              }`}
            >
              <div
                className={`w-8 h-8 rounded-control flex items-center justify-center shrink-0 ${
                  selectedAction === "exit_app"
                    ? "bg-status-error text-content-inverse"
                    : "bg-background-subtle text-status-error"
                }`}
              >
                <Power className="w-4 h-4" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between gap-2">
                  <span className="text-small font-semibold text-content-primary">
                    {t("close_dialog.exit_title")}
                  </span>
                  <span className="text-[11px] font-medium px-2 py-0.5 rounded-pill bg-status-error/10 text-status-error shrink-0">
                    {t("close_dialog.exit_badge")}
                  </span>
                </div>
                <p className="mt-1 mb-0 text-caption text-content-secondary leading-relaxed">
                  {t("close_dialog.exit_desc")}
                </p>
              </div>
            </div>
          </div>
          <div className="pt-2 border-t border-border flex items-center justify-between">
            <label className="flex items-center gap-2 cursor-pointer select-none">
              <input
                type="checkbox"
                id="rememberCloseChoice"
                checked={rememberChoice}
                onChange={(e) => setRememberChoice(e.target.checked)}
                className="w-4 h-4 rounded border-border text-primary focus:ring-primary cursor-pointer"
              />
              <span className="text-caption text-content-secondary">
                {t("close_dialog.remember_choice")}
              </span>
            </label>
          </div>
        </div>
      }
    />
  );
};

export default CloseAppDialog;
