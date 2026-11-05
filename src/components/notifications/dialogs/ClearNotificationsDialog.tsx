import React from "react";
import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { CommonDialog } from "../../common";

interface ClearNotificationsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

export const ClearNotificationsDialog: React.FC<ClearNotificationsDialogProps> = ({
  isOpen,
  onClose,
  onConfirm,
}) => {
  const { t } = useTranslation("dashboard");

  return (
    <CommonDialog
      isOpen={isOpen}
      onClose={onClose}
      eyebrow={t("notifications_page.dialog_eyebrow")}
      titleIcon={<Trash2 className="w-5 h-5 text-status-error" />}
      title={t("notifications_page.dialog_title")}
      content={t("notifications_page.dialog_content")}
      backButton={{
        label: t("notifications_page.dialog_cancel"),
        onClick: onClose,
      }}
      rightButton={{
        label: t("notifications_page.dialog_confirm"),
        variant: "danger",
        onClick: () => {
          onClose();
          onConfirm();
        },
      }}
    />
  );
};

export default ClearNotificationsDialog;