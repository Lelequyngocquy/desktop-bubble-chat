import React from "react";
import { Cpu, Sparkles, XCircle, MousePointerClick, Menu, Info } from "lucide-react";
import { useTranslation } from "react-i18next";
import { CommonLargeCard } from "../../common/index";

export const ResourceOptimizationCard: React.FC = () => {
  const { t } = useTranslation("help");

  return (
    <CommonLargeCard
      icon={<Cpu className="w-4 h-4 text-primary" />}
      title={t("optimize_title")}
      subTitle={
        <span className="px-2.5 py-0.5 bg-status-successBg text-status-success border border-status-successBorder rounded-pill text-caption font-semibold inline-flex items-center gap-1 shadow-2xs">
          <Sparkles className="w-3 h-3 text-status-success" />
          {t("recommended_badge")}
        </span>
      }
      className="w-full shrink-0 p-6 justify-between gap-5 select-none"
    >
      <div className="flex flex-col gap-4">
        {/* Benefit summary banner */}
        <div className="p-3.5 bg-status-successBg rounded-card-sm border border-status-successBorder flex items-start gap-3">
          <div className="w-8 h-8 rounded-panel bg-status-successBg text-status-success flex items-center justify-center shrink-0 mt-0.5">
            <Cpu className="w-4 h-4" />
          </div>
          <div className="flex flex-col gap-1 text-small text-status-success leading-relaxed">
            <span className="font-semibold text-status-success">
              {t("banner_title")}
            </span>
            <span>
              {t("banner_desc")}
            </span>
          </div>
        </div>

        {/* 3 Step guide for system tray usage */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-small text-content-secondary leading-relaxed">
          {/* Step 1 */}
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-primary-subtle text-primary-foreground flex items-center justify-center shrink-0">
                <XCircle className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                {t("step1_title")}
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              {t("step1_desc")}
            </span>
          </div>

          {/* Step 2 */}
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-primary-subtle text-primary-foreground flex items-center justify-center shrink-0">
                <MousePointerClick className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                {t("step2_title")}
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              {t("step2_desc")}
            </span>
          </div>

          {/* Step 3 */}
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-primary-subtle text-primary-foreground flex items-center justify-center shrink-0">
                <Menu className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                {t("step3_title")}
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              {t("step3_desc")}
            </span>
          </div>
        </div>
      </div>

      {/* Footer hint */}
      <div className="p-3 bg-background-app/60 rounded-panel flex items-start gap-2 text-caption text-content-secondary">
        <Info className="w-3.5 h-3.5 shrink-0 text-primary mt-0.5" />
        <span>
          {t("tray_tip")}
        </span>
      </div>
    </CommonLargeCard>
  );
};

export default ResourceOptimizationCard;