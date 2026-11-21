import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import enCommon from "@locales/en/common.json";
import viCommon from "@locales/vi/common.json";
import enDashboard from "@locales/en/dashboard.json";
import viDashboard from "@locales/vi/dashboard.json";
import enPerformance from "@locales/en/performance.json";
import viPerformance from "@locales/vi/performance.json";
import enSettings from "@locales/en/settings.json";
import viSettings from "@locales/vi/settings.json";
import enHelp from "@locales/en/help.json";
import viHelp from "@locales/vi/help.json";

export const defaultNS = "common";
export const resources = {
  vi: {
    common: viCommon,
    dashboard: viDashboard,
    performance: viPerformance,
    settings: viSettings,
    help: viHelp,
  },
  en: {
    common: enCommon,
    dashboard: enDashboard,
    performance: enPerformance,
    settings: enSettings,
    help: enHelp,
  },
} as const;

function detectInitialLanguage(): "en" | "vi" {
  try {
    if (typeof localStorage !== "undefined") {
      const raw = localStorage.getItem("desktop_bubble_chat_settings");
      if (raw) {
        const parsed = JSON.parse(raw);
        const saved = parsed?.state?.language;
        if (saved === "vi" || saved === "en") return saved;
      }
    }
  } catch (err) {
    console.warn("[i18n] Could not read saved locale from storage:", err);
  }
  return "en";
}

const initialLanguage = detectInitialLanguage();
i18n.use(initReactI18next).init({
  resources,
  lng: initialLanguage,
  fallbackLng: "en",
  defaultNS,
  interpolation: {
    escapeValue: false,
  },
});
i18n.on("languageChanged", (lng) => {
  if (typeof document !== "undefined") {
    document.documentElement.lang = lng;
  }
});

export default i18n;
