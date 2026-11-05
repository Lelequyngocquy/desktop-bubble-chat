import React from "react";
import { Lightbulb } from "lucide-react";
import { useTranslation } from "react-i18next";

export const HomeFooter: React.FC = () => {
  const { t } = useTranslation("dashboard");

  return (
    <footer className="shrink-0 pt-1 flex items-center justify-between text-caption text-content-secondary select-none flex-wrap gap-2">
      <div className="flex items-center gap-1.5">
        <Lightbulb className="w-[13px] h-[13px] text-content-secondary shrink-0" />
        <span>
          {t("footer_tip")}
        </span>
      </div>
    </footer>
  );
};

export default HomeFooter;