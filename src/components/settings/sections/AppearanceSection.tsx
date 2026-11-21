import React from "react";
import { useTranslation } from "react-i18next";
import { Palette, Sun, Moon, Laptop } from "lucide-react";
import { useSettingsStore, ThemeMode } from "../../../stores/settingsStore";
import { CommonLargeCard, OptionCardGrid, type OptionCardItem } from "../../common";

export const AppearanceSection: React.FC = () => {
  const { t } = useTranslation(["settings"]);
  const themeMode = useSettingsStore((s) => s.themeMode);
  const setThemeMode = useSettingsStore.getState().setThemeMode;
  const options: Array<OptionCardItem<ThemeMode>> = [
    {
      value: "light",
      label: t("settings:appearance.light"),
      description: t("settings:appearance.light_desc"),
      icon: <Sun className="w-4 h-4 text-amber-500" />,
      iconBoxed: true,
    },
    {
      value: "dark",
      label: t("settings:appearance.dark"),
      description: t("settings:appearance.dark_desc"),
      icon: <Moon className="w-4 h-4 text-indigo-400" />,
      iconBoxed: true,
    },
    {
      value: "system",
      label: t("settings:appearance.system"),
      description: t("settings:appearance.system_desc"),
      icon: <Laptop className="w-4 h-4 text-primary" />,
      iconBoxed: true,
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
        <OptionCardGrid options={options} value={themeMode} onChange={setThemeMode} columns={3} />
      </div>
    </CommonLargeCard>
  );
};

export default AppearanceSection;
