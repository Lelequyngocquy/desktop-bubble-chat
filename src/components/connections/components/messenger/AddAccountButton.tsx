import React from "react";
import { Plus, ShieldAlert } from "lucide-react";

interface AddAccountButtonProps {
  count: number;
  maxCount: number;
  onAdd: () => void;
}

/**
 * Component nút thêm tài khoản Messenger (Single Responsibility)
 * Xử lý giới hạn tối đa 5 tài khoản và hiển thị tiến độ (X/5).
 */
export const AddAccountButton: React.FC<AddAccountButtonProps> = ({
  count,
  maxCount,
  onAdd,
}) => {
  const isMaxReached = count >= maxCount;

  if (isMaxReached) {
    return (
      <div className="w-full py-3 px-4 rounded-card-sm bg-background-subtle border border-border flex items-center justify-center gap-2 text-small font-medium text-content-secondary select-none">
        <ShieldAlert className="w-4 h-4 text-status-warning" />
        <span>Đã đạt giới hạn tối đa {maxCount} tài khoản Messenger ({count}/{maxCount})</span>
      </div>
    );
  }

  return (
    <button
      type="button"
      onClick={onAdd}
      className="w-full py-3 px-4 rounded-card-sm border-2 border-dashed border-border hover:border-primary bg-background-surface hover:bg-primary-subtle text-primary flex items-center justify-center gap-2 text-body font-semibold transition-all cursor-pointer shadow-xs active:scale-[0.99] group"
    >
      <div className="w-5 h-5 rounded-pill bg-primary-subtle group-hover:bg-primary flex items-center justify-center transition-colors">
        <Plus className="w-3.5 h-3.5 text-primary group-hover:text-content-inverse transition-colors" />
      </div>
      <span>Thêm tài khoản Messenger</span>
      <span className="text-caption font-normal px-2 py-0.5 rounded-pill bg-primary-subtle text-primary">
        {count}/{maxCount}
      </span>
    </button>
  );
};

export default AddAccountButton;
