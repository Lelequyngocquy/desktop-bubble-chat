import React from "react";
import { useTranslation } from "react-i18next";
import { Keyboard } from "lucide-react";
import { SwitchButton, CommonLargeCard } from "../../common";

interface GlobalShortcutSectionProps {
  isGlobalShortcutEnabled: boolean;
  onToggleGlobalShortcut: () => void | Promise<void>;
}

export const GlobalShortcutSection: React.FC<GlobalShortcutSectionProps> = ({
  isGlobalShortcutEnabled,
  onToggleGlobalShortcut,
}) => {
  const { t } = useTranslation(["settings"]);
  return (
    <CommonLargeCard
      icon={<Keyboard className="w-4 h-4 text-primary" />}
      title={t("settings:shortcuts.title")}
    >
      <div className="flex items-center justify-between gap-4 py-1">
        <div className="flex flex-col gap-0.5">
          <div className="flex items-center gap-2">
            <span className="text-body font-medium text-content-primary">
              {t("settings:shortcuts.toggle_label")}
            </span>
            <span className="px-2 py-0.5 bg-background-app border border-border rounded-xs text-caption font-mono text-content-primary font-semibold">
              Ctrl + Shift + M
            </span>
          </div>
          <span className="text-caption text-content-secondary">
            {t("settings:shortcuts.desc")}
          </span>
        </div>
        <SwitchButton
          checked={isGlobalShortcutEnabled}
          onChange={onToggleGlobalShortcut}
        />
      </div>
    </CommonLargeCard>
  );
};

export default GlobalShortcutSection;
