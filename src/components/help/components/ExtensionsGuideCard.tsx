import React from "react";
import {
  Puzzle,
  FolderOpen,
  Info,
  AlertTriangle,
  RotateCcw,
  FolderPlus,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { CommonLargeCard } from "../../common/index";

export const ExtensionsGuideCard: React.FC = () => {
  const { t } = useTranslation("help");
  const openFolder = (index: number) => {
    invoke("open_profile_extensions_folder", { accountIndex: index }).catch(
      () => {},
    );
  };
  return (
    <CommonLargeCard
      icon={<Puzzle className="w-4 h-4 text-primary" />}
      title={t("ext_title")}
      subTitle={
        <span className="px-2 py-0.5 bg-background-subtle text-content-secondary border border-border rounded-pill text-caption font-semibold">
          Windows only
        </span>
      }
      className="w-full shrink-0 p-6 justify-between gap-5 select-none"
    >
      <div className="flex flex-col gap-4">
        {/* What it is */}
        <p className="text-small text-content-secondary leading-relaxed">
          {t("ext_desc")}
        </p>
        {/* 3-step guide */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-small text-content-secondary leading-relaxed">
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-background-surface border border-border text-primary flex items-center justify-center shrink-0">
                <FolderOpen className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                {t("ext_step1_title")}
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              {t("ext_step1_desc")}
            </span>
            {/* Quick-open buttons for each account slot */}
            <div className="flex flex-wrap gap-1.5 mt-1">
              {[1, 2, 3, 4, 5].map((i) => (
                <button
                  key={i}
                  onClick={() => openFolder(i)}
                  className="px-2 py-0.5 rounded-pill bg-background-surface border border-border text-caption text-primary hover:bg-background-subtle transition-colors cursor-pointer"
                >
                  #{i}
                </button>
              ))}
            </div>
          </div>
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-background-surface border border-border text-primary flex items-center justify-center shrink-0">
                <FolderPlus className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                {t("ext_step2_title")}
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              {t("ext_step2_desc")}
            </span>
          </div>
          <div className="p-3.5 bg-background-subtle rounded-card-sm border border-border flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-avatar-sm h-avatar-sm rounded-control bg-background-surface border border-border text-primary flex items-center justify-center shrink-0">
                <RotateCcw className="w-4 h-4" />
              </div>
              <span className="font-semibold text-content-primary">
                {t("ext_step3_title")}
              </span>
            </div>
            <span className="text-caption text-content-secondary">
              {t("ext_step3_desc")}
            </span>
          </div>
        </div>
        {/* Disable hint */}
        <div className="bg-background-app/60 rounded-panel flex items-center gap-2 text-caption text-content-secondary">
          <Info className="w-3.5 h-3.5 shrink-0 text-primary" />
          <span>{t("ext_disable_hint")}</span>
        </div>

        {/* Security warning */}
        <div className="bg-background-app/60 rounded-panel flex items-center gap-2 text-caption text-status-warning">
          <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
          <span>{t("ext_security_warn")}</span>
        </div>
      </div>
    </CommonLargeCard>
  );
};

export default ExtensionsGuideCard;
