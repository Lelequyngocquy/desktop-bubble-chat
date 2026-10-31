import React from "react";
import { CheckCheck, Trash2 } from "lucide-react";

interface NotificationsHeaderActionsProps {
  hasNotifications: boolean;
  onMarkAllRead: () => void;
  onOpenClearDialog: () => void;
}

export const NotificationsHeaderActions: React.FC<NotificationsHeaderActionsProps> = ({
  hasNotifications,
  onMarkAllRead,
  onOpenClearDialog,
}) => {
  if (!hasNotifications) {
    return null;
  }

  return (
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={onMarkAllRead}
        className="px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-lg text-[12px] font-medium text-[#1E293B] hover:bg-[#F8FAFC] flex items-center gap-1.5 transition-colors cursor-pointer shadow-xs"
      >
        <CheckCheck className="w-3.5 h-3.5 text-[#0866FF]" />
        <span>Đọc tất cả</span>
      </button>
      <button
        type="button"
        onClick={onOpenClearDialog}
        className="px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-lg text-[12px] font-medium text-[#DC2626] hover:bg-[#FEECEE] flex items-center gap-1.5 transition-colors cursor-pointer shadow-xs"
      >
        <Trash2 className="w-3.5 h-3.5" />
        <span>Xóa lịch sử</span>
      </button>
    </div>
  );
};
