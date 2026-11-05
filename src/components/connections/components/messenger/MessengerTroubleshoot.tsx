import React from "react";
import { HelpCircle, ArrowRight } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TROUBLESHOOT_STEPS } from "../../../../config";
import { CommonLargeCard } from "../../../common/index";
import { useNavigationStore } from "../../../../stores/navigationStore";

export const MessengerTroubleshoot: React.FC = () => {
  const { t } = useTranslation("dashboard");
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);

  return (
    <CommonLargeCard
      icon={<HelpCircle className="w-4 h-4 text-primary" />}
      title={t("connections_page.troubleshoot_title")}
      linkTitle={
        <button
          type="button"
          onClick={() => setActiveTab("help")}
          className="flex items-center gap-1 text-small font-medium text-primary-foreground hover:underline cursor-pointer"
        >
          <span>{t("connections_page.troubleshoot_help_link")}</span>
          <ArrowRight className="w-3.5 h-3.5" />
        </button>
      }
      className="shrink-0"
    >
      <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
        {TROUBLESHOOT_STEPS.map((step) => (
          <div key={step.num} className="flex items-start gap-2.5">
            <div className="w-5 h-5 rounded-pill bg-background-app text-content-secondary font-semibold text-caption flex items-center justify-center shrink-0 mt-0.5">
              {step.num}
            </div>
            <div className="flex flex-col">
              <span className="text-body font-medium text-content-primary">
                {t(`connections_page.troubleshoot_step_${step.num}_title`, step.title)}
              </span>
              <span className="text-caption text-content-secondary leading-relaxed">
                {t(`connections_page.troubleshoot_step_${step.num}_desc`, step.desc)}
              </span>
            </div>
          </div>
        ))}
      </div>
    </CommonLargeCard>
  );
};

export default MessengerTroubleshoot;