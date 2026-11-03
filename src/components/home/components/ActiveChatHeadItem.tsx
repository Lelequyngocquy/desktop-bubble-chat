import React from "react";
import { X, Pin } from "lucide-react";
import { Conversation } from "../../../types/conversation";
import { getAccountThemeColor } from "../../../constants/accountColors";

interface ActiveChatHeadItemProps {
  item: Conversation;
  onOpen: (item: Conversation) => void;
  onTogglePin: (id: string) => void;
  onRemove: (id: string) => void;
}

export const ActiveChatHeadItem: React.FC<ActiveChatHeadItemProps> = ({
  item,
  onOpen,
  onTogglePin,
  onRemove,
}) => {
  const color = getAccountThemeColor(item.accountIndex || 1);

  return (
    <div
      onClick={() => onOpen(item)}
      className="h-[52px] flex items-center gap-3 py-1 px-2 -mx-2 rounded-lg hover:bg-[#F8FAFC] cursor-pointer group transition-colors"
    >
      {/* Hủy bỏ avatar: thay bằng chấm tròn chỉ báo màu tài khoản Messenger */}
      <span
        className="w-2.5 h-2.5 rounded-full shrink-0 ml-1"
        style={{ backgroundColor: color.hex }}
        title={`Messenger ${item.accountIndex || 1}`}
      />

      <div className="flex-1 min-w-0 flex flex-col gap-0.5">
        <div className="flex items-center gap-1.5">
          <span className="text-[12.5px] font-semibold text-[#202733] truncate">
            {item.name}
          </span>
          {item.isPinned && (
            <Pin className="w-3 h-3 text-[#0866FF] fill-current shrink-0 -rotate-45" />
          )}
          {item.unreadCount > 0 && (
            <span className="h-[16px] min-w-[16px] px-1 rounded-full bg-[#0866FF] text-white text-[9.5px] font-bold flex items-center justify-center shrink-0 leading-none">
              {item.unreadCount}
            </span>
          )}
        </div>
        <p className="text-[11px] text-[#6B7483] truncate m-0 leading-tight">
          {item.lastMessage}
        </p>
      </div>

      <div className="flex items-center gap-1 shrink-0">
        <span className="text-[10.5px] text-[#929AA7] mr-0.5">
          {item.timestamp}
        </span>

        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onTogglePin(item.id);
          }}
          title={item.isPinned ? "Bỏ ghim" : "Ghim lên đầu (tối đa 6)"}
          className={`w-6 h-6 flex items-center justify-center rounded-[4px] transition-all cursor-pointer ${
            item.isPinned
              ? "text-[#0866FF] hover:bg-[#EAF2FF]"
              : "opacity-0 group-hover:opacity-100 text-[#6B7483] hover:text-[#0866FF] hover:bg-[#F1F5F9]"
          }`}
        >
          <Pin
            className={`w-3.5 h-3.5 ${item.isPinned ? "fill-current -rotate-45" : ""}`}
          />
        </button>

        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onRemove(item.id);
          }}
          title="Ẩn chat head"
          className="opacity-0 group-hover:opacity-100 w-6 h-6 flex items-center justify-center text-[#6B7483] hover:text-[#E81123] hover:bg-[#FEECEE] rounded-[4px] transition-all cursor-pointer"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};
