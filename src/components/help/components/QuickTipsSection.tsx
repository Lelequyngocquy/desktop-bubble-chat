import React from "react";
import { Sparkles, MousePointer, Keyboard } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TipCard } from "./TipCard";
import { CommonLargeCard } from "../../common/index";

export const QuickTipsSection: React.FC = () => {
  const { t } = useTranslation("help");

  return (
    <CommonLargeCard
      icon={<Sparkles className="w-4 h-4 text-primary" />}
      title={t("quick_tips_title")}
    >
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <TipCard
          icon={<MousePointer className="w-5 h-5" />}
          title={t("hover_title")}
          description={t("hover_desc")}
        />
        <TipCard
          icon={<Keyboard className="w-5 h-5" />}
          title={t("shortcut_title")}
          badge="Ctrl + Shift + M"
          description={t("shortcut_desc")}
        />
      </div>
    </CommonLargeCard>
  );
};

export default QuickTipsSection;