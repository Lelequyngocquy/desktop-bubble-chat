import React from "react";
import { Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";

export const RouteLoadingFallback: React.FC = () => {
  const { t } = useTranslation("common");
  return (
    <div
      className="flex-1 min-w-0 min-h-0 h-full flex items-center justify-center bg-background-workspace select-none"
      role="status"
      aria-live="polite"
    >
      <div className="flex items-center gap-3 px-5 py-3.5 bg-background-surface border border-border rounded-panel shadow-xs">
        <Loader2 className="w-5 h-5 animate-spin text-primary shrink-0" />
        <span className="text-small font-medium text-content-secondary">
          {t("loading_workspace")}
        </span>
      </div>
    </div>
  );
};

export default RouteLoadingFallback;
