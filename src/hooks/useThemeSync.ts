import { useEffect } from "react";
import { useSettingsStore, applyThemeMode } from "../stores/settingsStore";

/**
 * Synchronizes application dark/light theme with user settings and Windows OS preference
 */
export function useThemeSync() {
  const themeMode = useSettingsStore((s) => s.themeMode);

  useEffect(() => {
    applyThemeMode(themeMode);
    if (themeMode === "system" && typeof window !== "undefined") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const handleChange = () => {
        applyThemeMode("system");
      };
      if (mediaQuery.addEventListener) {
        mediaQuery.addEventListener("change", handleChange);
      } else {
        mediaQuery.addListener(handleChange);
      }
      return () => {
        if (mediaQuery.removeEventListener) {
          mediaQuery.removeEventListener("change", handleChange);
        } else {
          mediaQuery.removeListener(handleChange);
        }
      };
    }
  }, [themeMode]);
}
