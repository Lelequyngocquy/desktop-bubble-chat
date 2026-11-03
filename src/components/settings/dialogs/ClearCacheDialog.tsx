import React from "react";
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
  return (
    <CommonDialog
      isOpen={isOpen}
      onClose={onClose}
      eyebrow="XÁC NHẬN DỌN DẸP"
      titleIcon={<Trash2 className="w-5 h-5 text-status-error" />}
      title="Xóa bộ nhớ đệm WebView2"
      content="Bạn có chắc chắn muốn xóa bộ nhớ đệm WebView2? Thao tác này sẽ giải phóng dung lượng ổ đĩa và làm mới hoàn toàn session dữ liệu của trình duyệt nhúng."
      widget={
        <div className="p-3 bg-status-errorBg rounded-panel border border-status-errorBorder text-small text-status-error leading-relaxed">
          Lưu ý: Sau khi xóa bộ nhớ đệm, các phiên đăng nhập webview có thể
          cần tải lại tài nguyên trong lần mở kế tiếp.
        </div>
      }
      backButton={{
        label: "Hủy bỏ",
        onClick: onClose,
      }}
      rightButton={{
        label: "Xác nhận xóa",
        variant: "danger",
        loading: isClearing,
        onClick: onConfirm,
      }}
    />
  );
};

export default ClearCacheDialog;
