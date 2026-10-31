import React from "react";
import { Trash2 } from "lucide-react";
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
  return (
    <CommonDialog
      isOpen={isOpen}
      onClose={onClose}
      eyebrow="XÁC NHẬN XÓA"
      titleIcon={<Trash2 className="w-5 h-5 text-[#DC2626]" />}
      title="Xóa lịch sử thông báo"
      content="Bạn có chắc chắn muốn xóa toàn bộ danh sách thông báo? Thao tác này sẽ làm sạch các thông báo tin nhắn đã ghi nhận trên màn hình này."
      backButton={{
        label: "Hủy bỏ",
        onClick: onClose,
      }}
      rightButton={{
        label: "Xác nhận xóa",
        variant: "danger",
        onClick: () => {
          onClose();
          onConfirm();
        },
      }}
    />
  );
};
