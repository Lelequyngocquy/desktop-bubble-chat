import React from "react";
import { useTranslation } from "react-i18next";
import { Languages, Check } from "lucide-react";
import { useSettingsStore, AppLanguage } from "../../../stores/settingsStore";
import { CommonLargeCard } from "../../common";

export const LanguageSelectionSection: React.FC = () => {
  const { t } = useTranslation(["settings", "common"]);
  const currentLanguage = useSettingsStore((s) => s.language);
  const setLanguage = useSettingsStore((s) => s.setLanguage);
  const languages: Array<{ code: AppLanguage; label: string; flag: string; desc: string }> = [
    { code: "vi", label: "Tiếng Việt", flag: "🇻🇳", desc: t("settings:language.vietnamese_desc") },
    { code: "en", label: "English", flag: "🇺🇸", desc: t("settings:language.english_desc") },
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
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
          {languages.map((lang) => {
            const isSelected = currentLanguage === lang.code;
            return (
              <button
                key={lang.code}
                type="button"
                onClick={() => setLanguage(lang.code)}
                className={`flex items-center justify-between p-3.5 rounded-control border transition-all text-left cursor-pointer ${
                  isSelected
                    ? "bg-primary/10 border-primary shadow-xs ring-1 ring-primary/20"
                    : "bg-background-surface hover:bg-background-subtle border-border hover:border-border"
                }`}
              >
                <div className="flex items-center gap-3">
                  <span className="text-xl select-none">{lang.flag}</span>
                  <div className="flex flex-col">
                    <span className={`text-body leading-tight ${isSelected ? "font-semibold text-primary" : "font-medium text-content-primary"}`}>
                      {lang.label}
                    </span>
                    <span className="text-caption text-content-muted leading-tight mt-0.5">
                      {lang.desc}
                    </span>
                  </div>
                </div>
                {isSelected && (
                  <div className="w-5 h-5 rounded-full bg-primary text-white flex items-center justify-center shrink-0">
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

export default LanguageSelectionSection;