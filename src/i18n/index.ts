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
import enTray from "@locales/en/tray.json";
import viTray from "@locales/vi/tray.json";

export const defaultNS = "common";
export const resources = {
  vi: {
    common: viCommon,
    dashboard: viDashboard,
    performance: viPerformance,
    settings: viSettings,
    help: viHelp,
    tray: viTray,
  },
  en: {
    common: enCommon,
    dashboard: enDashboard,
    performance: enPerformance,
    settings: enSettings,
    help: enHelp,
    tray: enTray,
  },
} as const;

// Initialize i18next instance
i18n.use(initReactI18next).init({
  resources,
  lng: "en",
  fallbackLng: "en",
  defaultNS,
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
