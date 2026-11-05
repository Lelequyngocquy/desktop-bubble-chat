import React from "react";
import { useTranslation } from "react-i18next";
import { Palette, Sun, Moon, Laptop, Check } from "lucide-react";
import { useSettingsStore, ThemeMode } from "../../../stores/settingsStore";
import { CommonLargeCard } from "../../common";

export const AppearanceSection: React.FC = () => {
  const { t } = useTranslation(["settings"]);
  const themeMode = useSettingsStore((s) => s.themeMode);
  const setThemeMode = useSettingsStore((s) => s.setThemeMode);
  const options: Array<{ mode: ThemeMode; label: string; desc: string; icon: React.ReactNode }> = [
    {
      mode: "light",
      label: t("settings:appearance.light"),
      desc: t("settings:appearance.light_desc"),
      icon: <Sun className="w-4 h-4 text-amber-500" />,
    },
    {
      mode: "dark",
      label: t("settings:appearance.dark"),
      desc: t("settings:appearance.dark_desc"),
      icon: <Moon className="w-4 h-4 text-indigo-400" />,
    },
    {
      mode: "system",
      label: t("settings:appearance.system"),
      desc: t("settings:appearance.system_desc"),
      icon: <Laptop className="w-4 h-4 text-primary" />,
    },
  ];
  return (
    <CommonLargeCard
      icon={<Palette className="w-4 h-4 text-primary" />}
      title={t("settings:appearance.title")}
    >
      <div className="flex flex-col gap-3">
        <p className="text-small text-content-secondary m-0">
          {t("settings:appearance.description")}
        </p>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-1">
          {options.map((opt) => {
            const isSelected = themeMode === opt.mode;
            return (
              <button
                key={opt.mode}
                type="button"
                onClick={() => setThemeMode(opt.mode)}
                className={`flex items-center justify-between p-3.5 rounded-control border transition-all text-left cursor-pointer ${
                  isSelected
                    ? "bg-primary/10 border-primary shadow-xs ring-1 ring-primary/20"
                    : "bg-background-surface hover:bg-background-subtle border-border hover:border-border"
                }`}
              >
                <div className="flex items-center gap-3">
                  <div className={`w-8 h-8 rounded-panel flex items-center justify-center shrink-0 ${
                    isSelected ? "bg-primary/15" : "bg-background-subtle"
                  }`}>
                    {opt.icon}
                  </div>
                  <div className="flex flex-col">
                    <span className={`text-body leading-tight ${isSelected ? "font-semibold text-primary" : "font-medium text-content-primary"}`}>
                      {opt.label}
                    </span>
                    <span className="text-caption text-content-muted leading-tight mt-0.5">
                      {opt.desc}
                    </span>
                  </div>
                </div>
                {isSelected && (
                  <div className="w-5 h-5 rounded-full bg-primary text-white flex items-center justify-center shrink-0 ml-2">
                    <Check className="w-3 h-3 stroke-[3]" />
                  </div>
                )}
              </button>
            );
          })}
        </div>
      </div>
    </CommonLargeCard>
  );
};

export default AppearanceSection;
