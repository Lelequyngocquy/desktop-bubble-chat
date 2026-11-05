import React, { useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAppLifecycleStore } from "../../stores/appLifecycleStore";

export const DashboardLoadingOverlay: React.FC = () => {
  const { t } = useTranslation("common");
  const isDashboardReady = useAppLifecycleStore((s) => s.isDashboardReady);
  const statusMessage = useAppLifecycleStore((s) => s.statusMessage);
  const markDashboardReady = useAppLifecycleStore((s) => s.markDashboardReady);
  const [shouldRender, setShouldRender] = useState(!isDashboardReady);

  useEffect(() => {
    if (isDashboardReady) {
      const timer = setTimeout(() => {
        setShouldRender(false);
      }, 350);
      return () => clearTimeout(timer);
    } else {
      setShouldRender(true);
      const safetyTimer = setTimeout(() => {
        markDashboardReady();
      }, 4000);
      return () => clearTimeout(safetyTimer);
    }
  }, [isDashboardReady, markDashboardReady]);

  if (!shouldRender) return null;

  return (
    <div
      className={`absolute inset-0 z-50 flex items-center justify-center bg-background-app/85 backdrop-blur-xs select-none transition-opacity duration-300 ease-out ${
        isDashboardReady ? "opacity-0 pointer-events-none" : "opacity-100 pointer-events-auto"
      }`}
      aria-live="polite"
      aria-busy={!isDashboardReady}
    >
      <div className="bg-background-surface/95 border border-border shadow-window rounded-modal px-6 py-4 flex items-center gap-4 max-w-sm">
        <div className="relative flex items-center justify-center">
          <Loader2 className="w-6 h-6 animate-spin text-primary" />
        </div>
        <div className="flex flex-col min-w-0">
          <span className="text-nav font-semibold text-content-primary truncate">
            {statusMessage || t("overlay_initializing")}
          </span>
          <span className="text-small text-content-secondary mt-0.5">
            {t("overlay_stabilizing")}
          </span>
        </div>
      </div>
    </div>
  );
};

export default DashboardLoadingOverlay;