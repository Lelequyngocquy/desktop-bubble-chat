import React from "react";
import { useTranslation } from "react-i18next";
import { Trash2 } from "lucide-react";
import { CommonDialog } from "../../common";

interface ClearCacheDialogProps {
  isOpen: boolean;
  isClearing: boolean;
  onClose: () => void;
  onConfirm: () => void | Promise<void>;
}

export const ClearCacheDialog: React.FC<ClearCacheDialogProps> = ({
  isOpen,
  isClearing,
  onClose,
  onConfirm,
}) => {
  const { t } = useTranslation(["settings", "common"]);
  return (
    <CommonDialog
      isOpen={isOpen}
      onClose={onClose}
      eyebrow={t("common:confirm").toUpperCase()}
      titleIcon={<Trash2 className="w-5 h-5 text-status-error" />}
      title={t("settings:data_cache.dialog_title")}
      content={t("settings:data_cache.dialog_desc")}
      widget={
        <div className="p-3 bg-status-errorBg rounded-panel border border-status-errorBorder text-small text-status-error leading-relaxed">
          {t("settings:data_cache.warning")}
        </div>
      }
      backButton={{
        label: t("common:cancel"),
        onClick: onClose,
      }}
      rightButton={{
        label: t("common:delete"),
        variant: "danger",
        loading: isClearing,
        onClick: onConfirm,
      }}
    />
  );
};

export default ClearCacheDialog;
