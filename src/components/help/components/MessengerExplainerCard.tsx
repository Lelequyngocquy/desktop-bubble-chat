import React from "react";
import { PanelsTopLeft, Info } from "lucide-react";
import { useTranslation } from "react-i18next";
import { CommonLargeCard } from "../../common/index";

export const MessengerExplainerCard: React.FC = () => {
  const { t } = useTranslation("help");

  return (
    <CommonLargeCard
      icon={<PanelsTopLeft className="w-4 h-4 text-primary" />}
      title={t("multi_account_title")}
      className="w-full shrink-0 p-6 justify-between gap-5 select-none"
    >
      <div className="flex flex-col gap-4">
        <div className="space-y-3 text-small text-content-secondary leading-relaxed">
          <div className="p-3 bg-background-subtle rounded-panel border border-border/80 flex flex-col gap-1">
            <span className="font-semibold text-content-primary">
              {t("item1_title")}
            </span>
            <span>
              {t("item1_desc")}
            </span>
          </div>
          <div className="p-3 bg-background-subtle rounded-panel border border-border/80 flex flex-col gap-1">
            <span className="font-semibold text-content-primary">
              {t("item2_title")}
            </span>
            <span>
              {t("item2_desc")}
            </span>
          </div>
          <div className="p-3 bg-background-subtle rounded-panel border border-border/80 flex flex-col gap-1">
            <span className="font-semibold text-content-primary">
              {t("item3_title")}
            </span>
            <span>
              {t("item3_desc")}
            </span>
          </div>
        </div>
      </div>
      <div className="p-3 bg-background-app/50 rounded-panel flex items-start gap-2 text-caption text-content-secondary">
        <Info className="w-3.5 h-3.5 shrink-0 text-primary mt-0.5" />
        <span>
          {t("settings_hint")}
        </span>
      </div>
    </CommonLargeCard>
  );
};

export default MessengerExplainerCard;