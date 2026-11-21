import React from "react";
import { useTranslation } from "react-i18next";
import { Languages } from "lucide-react";
import { useSettingsStore, AppLanguage } from "../../../stores/settingsStore";
import { CommonLargeCard, OptionCardGrid, type OptionCardItem } from "../../common";

export const LanguageSelectionSection: React.FC = () => {
  const { t } = useTranslation(["settings", "common"]);
  const currentLanguage = useSettingsStore((s) => s.language);
  const setLanguage = useSettingsStore.getState().setLanguage;
  const languages: Array<OptionCardItem<AppLanguage>> = [
    {
      value: "vi",
      label: "Tiếng Việt",
      description: t("settings:language.vietnamese_desc"),
      icon: <span className="text-xl select-none">🇻🇳</span>,
    },
    {
      value: "en",
      label: "English",
      description: t("settings:language.english_desc"),
      icon: <span className="text-xl select-none">🇺🇸</span>,
    },
  ];
  return (
    <CommonLargeCard
      icon={<Languages className="w-4 h-4 text-primary" />}
      title={t("settings:language.title")}
    >
      <div className="flex flex-col gap-3">
        <p className="text-small text-content-secondary m-0">
          {t("settings:language.description")}
        </p>
        <OptionCardGrid options={languages} value={currentLanguage} onChange={setLanguage} columns={2} />
      </div>
    </CommonLargeCard>
  );
};

export default LanguageSelectionSection;
