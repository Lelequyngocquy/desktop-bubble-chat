import React from "react";
import { Trash2, ArrowUpRight } from "lucide-react";
import { Conversation } from "../../../types/conversation";
import { getAccountThemeColor } from "../../../constants/accountColors";

interface NotificationItemRowProps {
  item: Conversation;
  onOpenNotification: (item: Conversation) => void;
  onRemove: (id: string) => void;
}

export const NotificationItemRow: React.FC<NotificationItemRowProps> = ({
  item,
  onOpenNotification,
  onRemove,
}) => {
  const color = getAccountThemeColor(item.accountIndex || 1);
  const isUnread = (item.unreadCount || 0) > 0;

  return (
    <div
      onClick={() => onOpenNotification(item)}
      className={`py-3.5 px-3 flex items-center justify-between gap-3.5 rounded-xl transition-all cursor-pointer ${
        isUnread
          ? "bg-[#F0F7FF] hover:bg-[#E5F0FF]"
          : "hover:bg-[#F8FAFC]"
      }`}
    >
      <div className="flex items-center gap-3 min-w-0 flex-1">
        {/* Hủy bỏ avatar: thay bằng chấm tròn chỉ báo màu tài khoản Messenger */}
        <span
          className="w-2.5 h-2.5 rounded-full shrink-0"
          style={{ backgroundColor: color.hex }}
          title={`Messenger ${item.accountIndex || 1}`}
        />

        {/* Content */}
        <div className="flex-1 min-w-0 flex flex-col gap-0.5">
          <div className="flex items-center gap-2">
            <span
              className={`text-[13px] truncate ${
                isUnread ? "font-bold text-[#0F172A]" : "font-medium text-[#334155]"
              }`}
            >
              {item.name}
            </span>
          </div>
          <p
            className={`text-[12px] truncate m-0 ${
              isUnread ? "text-[#1E293B] font-medium" : "text-[#64748B]"
            }`}
          >
            {item.lastMessage}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-2 shrink-0">
        <span className="text-[11px] text-[#94A3B8]">{item.timestamp}</span>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onRemove(item.id);
          }}
          title="Xóa thông báo"
          className="w-7 h-7 flex items-center justify-center rounded-md text-[#94A3B8] hover:text-[#DC2626] hover:bg-[#FEECEE] transition-colors cursor-pointer"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
        <ArrowUpRight className="w-4 h-4 text-[#94A3B8] group-hover:text-[#0866FF]" />
      </div>
    </div>
  );
};
