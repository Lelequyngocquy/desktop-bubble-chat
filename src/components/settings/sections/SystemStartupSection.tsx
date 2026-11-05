import React from "react";
import { useTranslation } from "react-i18next";
import { Power } from "lucide-react";
import { SwitchButton, CommonLargeCard } from "../../common";

interface SystemStartupSectionProps {
  isAutoStart: boolean;
  isAlwaysOnTop: boolean;
  isNotificationsEnabled: boolean;
  onToggleAutoStart: () => void | Promise<void>;
  onToggleAlwaysOnTop: () => void | Promise<void>;
  onToggleNotifications: () => void | Promise<void>;
}

export const SystemStartupSection: React.FC<SystemStartupSectionProps> = ({
  isAutoStart,
  isAlwaysOnTop,
  isNotificationsEnabled,
  onToggleAutoStart,
  onToggleAlwaysOnTop,
  onToggleNotifications,
}) => {
  const { t } = useTranslation(["settings"]);
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
      </div>
    </CommonLargeCard>
  );
};

export default SystemStartupSection;
