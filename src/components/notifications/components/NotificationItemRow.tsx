import React from "react";
import { Trash2, ArrowUpRight } from "lucide-react";
import { Conversation } from "../../../types/conversation";
import { AvatarCircle } from "../../common";
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
      <div className="flex items-center gap-3.5 min-w-0 flex-1">
        {/* Avatar */}
        <div className="relative shrink-0">
          {item.avatarUrl ? (
            <img
              src={item.avatarUrl}
              alt={item.name}
              className="w-10 h-10 rounded-full object-cover border border-[#CBD5E1]"
            />
          ) : (
            <AvatarCircle
              size={40}
              text={item.avatarText || item.name.slice(0, 2).toUpperCase()}
              bg={isUnread ? "#DBEAFE" : "#F1F5F9"}
              textColor={isUnread ? "#1D4ED8" : "#475569"}
            />
          )}
          <span
            className="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-white"
            style={{ backgroundColor: color.hex }}
            title={`Tài khoản: Messenger ${item.accountIndex || 1}`}
          />
        </div>

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
            <span
              className="text-[10px] px-1.5 py-0.2 rounded text-white font-medium"
              style={{ backgroundColor: color.hex }}
            >
              M{item.accountIndex || 1}
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
