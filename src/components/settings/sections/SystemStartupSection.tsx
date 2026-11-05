import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Power } from "lucide-react";
import { SwitchButton, CommonLargeCard } from "../../common";

interface SystemStartupSectionProps {
  isAutoStart: boolean;
  isAlwaysOnTop: boolean;
  isNotificationsEnabled: boolean;
  isAutoTrimEnabled: boolean;
  onToggleAutoStart: () => void | Promise<void>;
  onToggleAlwaysOnTop: () => void | Promise<void>;
  onToggleNotifications: () => void | Promise<void>;
  onToggleAutoTrim: () => void | Promise<void>;
}

export const SystemStartupSection: React.FC<SystemStartupSectionProps> = ({
  isAutoStart,
  isAlwaysOnTop,
  isNotificationsEnabled,
  isAutoTrimEnabled,
  onToggleAutoStart,
  onToggleAlwaysOnTop,
  onToggleNotifications,
  onToggleAutoTrim,
}) => {
  const { t } = useTranslation(["settings"]);
  const [closeAction, setCloseAction] = useState<"ask" | "hide_to_tray" | "exit_app">(() => {
    const val = localStorage.getItem("dbc_close_window_action");
    if (val === "hide_to_tray" || val === "exit_app") return val;
    return "ask";
  });
  const handleSetCloseAction = (val: "ask" | "hide_to_tray" | "exit_app") => {
    setCloseAction(val);
    if (val === "ask") {
      localStorage.removeItem("dbc_close_window_action");
    } else {
      localStorage.setItem("dbc_close_window_action", val);
    }
  };
  const closeActionOptions: Array<{ id: "ask" | "hide_to_tray" | "exit_app"; label: string }> = [
    { id: "ask", label: t("settings:startup.close_action_ask") },
    { id: "hide_to_tray", label: t("settings:startup.close_action_hide") },
    { id: "exit_app", label: t("settings:startup.close_action_exit") },
  ];

  return (
    <CommonLargeCard
      icon={<Power className="w-4 h-4 text-primary" />}
      title={t("settings:startup.title")}
    >
      <div className="flex flex-col divide-y divide-border">
        <div className="py-3 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-0.5">
            <span className="text-body font-medium text-content-primary">
              {t("settings:startup.autostart")}
            </span>
            <span className="text-caption text-content-secondary">
              {t("settings:startup.autostart_desc")}
            </span>
          </div>
          <SwitchButton
            checked={isAutoStart}
            onChange={onToggleAutoStart}
          />
        </div>

        <div className="py-3 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-0.5">
            <span className="text-body font-medium text-content-primary">
              {t("settings:startup.always_on_top")}
            </span>
            <span className="text-caption text-content-secondary">
              {t("settings:startup.always_on_top_desc")}
            </span>
          </div>
          <SwitchButton
            checked={isAlwaysOnTop}
            onChange={onToggleAlwaysOnTop}
          />
        </div>

        <div className="py-3 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-0.5">
            <span className="text-body font-medium text-content-primary">
              {t("settings:startup.notifications")}
            </span>
            <span className="text-caption text-content-secondary">
              {t("settings:startup.notifications_desc")}
            </span>
          </div>
          <SwitchButton
            checked={isNotificationsEnabled}
            onChange={onToggleNotifications}
          />
        </div>

        <div className="py-3 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-0.5">
            <span className="text-body font-medium text-content-primary">
              {t("settings:startup.auto_trim")}
            </span>
            <span className="text-caption text-content-secondary">
              {t("settings:startup.auto_trim_desc")}
            </span>
          </div>
          <SwitchButton
            checked={isAutoTrimEnabled}
            onChange={onToggleAutoTrim}
          />
        </div>

        <div className="py-3 flex flex-col gap-2">
          <div className="flex flex-col gap-0.5">
            <span className="text-body font-medium text-content-primary">
              {t("settings:startup.close_action")}
            </span>
            <span className="text-caption text-content-secondary">
              {t("settings:startup.close_action_desc")}
            </span>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 pt-1">
            {closeActionOptions.map((opt) => (
              <button
                key={opt.id}
                type="button"
                onClick={() => handleSetCloseAction(opt.id)}
                className={`p-2.5 rounded-control border text-center transition-all cursor-pointer ${
                  closeAction === opt.id
                    ? "border-primary bg-primary-subtle text-primary font-semibold shadow-xs ring-1 ring-primary/20"
                    : "border-border bg-background-surface text-content-secondary hover:border-border-strong hover:bg-background-subtle"
                }`}
              >
                <span className="text-small">{opt.label}</span>
              </button>
            ))}
          </div>
        </div>
      </div>
    </CommonLargeCard>
  );
};

export default SystemStartupSection;
