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
      <div className="w-full py-3 px-4 rounded-[10px] bg-[#F8FAFD] border border-[#E2E7EF] flex items-center justify-center gap-2 text-[12px] font-medium text-[#6B7483] select-none">
        <ShieldAlert className="w-4 h-4 text-[#D97706]" />
        <span>Đã đạt giới hạn tối đa {maxCount} tài khoản Messenger ({count}/{maxCount})</span>
      </div>
    );
  }

  return (
    <button
      type="button"
      onClick={onAdd}
      className="w-full py-3 px-4 rounded-[10px] border-2 border-dashed border-[#CBD5E1] hover:border-[#0866FF] bg-white hover:bg-[#F0F7FF] text-[#0866FF] flex items-center justify-center gap-2 text-[13px] font-semibold transition-all cursor-pointer shadow-xs active:scale-[0.99] group"
    >
      <div className="w-5 h-5 rounded-full bg-[#EAF2FF] group-hover:bg-[#0866FF] flex items-center justify-center transition-colors">
        <Plus className="w-3.5 h-3.5 text-[#0866FF] group-hover:text-white transition-colors" />
      </div>
      <span>Thêm tài khoản Messenger</span>
      <span className="text-[11px] font-normal px-2 py-0.5 rounded-full bg-[#EAF2FF] text-[#0866FF]">
        {count}/{maxCount}
      </span>
    </button>
  );
};
