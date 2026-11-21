import React, { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useShallow } from "zustand/react/shallow";
import { Trash2 } from "lucide-react";
import { CommonDialog } from "../../common";
import { useMessengerStore, formatMessengerName } from "../../../platforms/messenger";

interface ClearCacheDialogProps {
  isOpen: boolean;
  isClearing: boolean;
  onClose: () => void;
  onConfirm: (accountIndices: number[]) => void | Promise<void>;
}

export const ClearCacheDialog: React.FC<ClearCacheDialogProps> = ({
  isOpen,
  isClearing,
  onClose,
  onConfirm,
}) => {
  const { t } = useTranslation(["settings", "common"]);
  const accountIndices = useMessengerStore(
    useShallow((s) => s.accounts.map((a) => a.index)),
  );
  const accounts = useMemo(
    () =>
      accountIndices.map((index) => ({
        id: `messenger-${index.toString().padStart(2, "0")}`,
        index,
        name: formatMessengerName(index),
      })),
    [accountIndices],
  );
  const [selectedTarget, setSelectedTarget] = useState<"all" | number>("all");
  useEffect(() => {
    if (isOpen) {
      setSelectedTarget("all");
    }
  }, [isOpen]);
  const resolveTargetIndices = (): number[] => {
    if (selectedTarget === "all") {
      return accounts.length > 0 ? accounts.map((a) => a.index) : [1];
    }
    return [selectedTarget];
  };
  return (
    <CommonDialog
      isOpen={isOpen}
      onClose={() => {
        if (!isClearing) onClose();
      }}
      closeOnBackdropClick={!isClearing}
      closeOnEscape={!isClearing}
      showCloseButton={!isClearing}
      eyebrow={t("common:confirm").toUpperCase()}
      titleIcon={<Trash2 className="w-5 h-5 text-status-error" />}
      title={t("settings:data_cache.dialog_title")}
      content={t("settings:data_cache.dialog_desc")}
      widget={
        <div className="flex flex-col gap-3">
          <div className="flex flex-col gap-2">
            <span className="text-caption font-medium text-content-secondary">
              {t("settings:data_cache.target_label")}
            </span>
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                disabled={isClearing}
                onClick={() => setSelectedTarget("all")}
                className={`px-3 py-1.5 rounded-control text-caption font-semibold border transition-colors cursor-pointer disabled:opacity-60 ${
                  selectedTarget === "all"
                    ? "bg-primary/10 border-primary text-primary"
                    : "bg-background-subtle border-border text-content-secondary hover:text-content-primary"
                }`}
              >
                {t("settings:data_cache.all_accounts", { count: Math.max(1, accounts.length) })}
              </button>
              {accounts.map((acc) => (
                <button
                  key={acc.id}
                  type="button"
                  disabled={isClearing}
                  onClick={() => setSelectedTarget(acc.index)}
                  className={`px-3 py-1.5 rounded-control text-caption font-semibold border transition-colors cursor-pointer disabled:opacity-60 ${
                    selectedTarget === acc.index
                      ? "bg-primary/10 border-primary text-primary"
                      : "bg-background-subtle border-border text-content-secondary hover:text-content-primary"
                  }`}
                >
                  {acc.name}
                </button>
              ))}
            </div>
          </div>
          <div className="p-3 bg-status-errorBg rounded-panel border border-status-errorBorder text-small text-status-error leading-relaxed">
            {t("settings:data_cache.warning")}
          </div>
        </div>
      }
      backButton={{
        label: t("common:cancel"),
        disabled: isClearing,
        onClick: onClose,
      }}
      rightButton={{
        label: isClearing ? t("settings:data_cache.clearing") : t("common:delete"),
        variant: "danger",
        loading: isClearing,
        onClick: () => onConfirm(resolveTargetIndices()),
      }}
    />
  );
};

export default ClearCacheDialog;
