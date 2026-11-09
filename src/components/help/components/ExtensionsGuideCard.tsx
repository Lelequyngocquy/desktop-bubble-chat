import React from "react";
import { Puzzle, FolderOpen, AlertTriangle, RotateCcw, FolderPlus, Info } from "lucide-react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { CommonLargeCard, CommonHintLine } from "../../common/index";
import { useMessengerStore } from "../../../platforms/messenger";

export const ExtensionsGuideCard: React.FC = () => {
  const { t } = useTranslation("help");
  const accounts = useMessengerStore((s) => s.accounts);
  const openFolder = (index: number) => {
    invoke("open_profile_extensions_folder", { accountIndex: index }).catch(() => {});
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
            {/* Quick-open buttons — only for existing accounts */}
            <div className="flex flex-wrap gap-1.5 mt-1">
              {accounts.map((acc) => (
                <button
                  key={acc.index}
                  onClick={() => openFolder(acc.index)}
                  className="px-2 py-0.5 rounded-pill bg-background-surface border border-border text-caption text-primary hover:bg-background-subtle transition-colors cursor-pointer"
                >
                  #{acc.index}
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
        <CommonHintLine
          icon={<Info className="w-3.5 h-3.5" />}
          text={t("ext_disable_hint")}
        />
        {/* Security warning */}
        <CommonHintLine
          icon={<AlertTriangle className="w-3.5 h-3.5" />}
          text={t("ext_security_warn")}
          color="text-status-warning"
        />
      </div>
    </CommonLargeCard>
  );
};

export default ExtensionsGuideCard;
