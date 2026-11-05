import React from "react";
import { useTranslation } from "react-i18next";

export const ConnectionsFooter: React.FC = () => {
  const { t } = useTranslation("dashboard");

  return (
    <footer className="shrink-0 flex items-center justify-between pt-1 text-caption text-content-secondary select-none">
      <span>{t("connections_page.footer_version_hint")}</span>
    </footer>
  );
};

export default ConnectionsFooter;