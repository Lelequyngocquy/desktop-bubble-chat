import React from "react";
import { Pin, Trash2, Users } from "lucide-react";
import { Conversation } from "../../../types/conversation";
import { AvatarCircle } from "../../common";

interface RecentChatItemProps {
  item: Conversation;
  canPin: boolean;
  onOpenThread: (item: Conversation) => void;
  onPin: (id: string) => void;
  onRemove: (id: string) => void;
}

export const RecentChatItem: React.FC<RecentChatItemProps> = ({
  item,
  canPin,
  onOpenThread,
  onPin,
  onRemove,
}) => {
  return (
    <div className="py-3 flex items-center justify-between gap-3 hover:bg-[#F8FAFC] px-2 rounded-lg transition-colors">
      <div
        onClick={() => onOpenThread(item)}
        className="flex items-center gap-3 min-w-0 flex-1 cursor-pointer"
      >
        {item.avatarUrl ? (
          <img
            src={item.avatarUrl}
            alt={item.name}
            className="w-9 h-9 rounded-full object-cover shrink-0 border border-[#CBD5E1]"
          />
        ) : (
          <AvatarCircle
            size={36}
            icon={item.isGroup ? <Users className="w-4 h-4 text-[#7762AC]" /> : undefined}
            text={!item.isGroup ? item.avatarText || item.name.slice(0, 2).toUpperCase() : undefined}
            bg={item.isGroup ? "#F2EEFF" : "#EAF2FF"}
            textColor={item.isGroup ? "#7762AC" : "#0759DB"}
          />
        )}
        <div className="min-w-0 flex flex-col">
          <span className="text-[13px] font-medium text-[#1E293B] truncate">
            {item.name}
          </span>
          <span className="text-[11px] text-[#64748B] truncate">
            {item.lastMessage}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-2 shrink-0">
        <span className="text-[11px] text-[#94A3B8] mr-1">{item.timestamp}</span>
        <button
          type="button"
          onClick={() => onPin(item.id)}
          disabled={!canPin}
          title={!canPin ? "Đã đạt tối đa 6 ghim" : "Ghim"}
          className="w-7 h-7 flex items-center justify-center rounded-md text-[#64748B] hover:text-[#0866FF] hover:bg-[#EAF2FF] disabled:opacity-30 disabled:cursor-not-allowed transition-colors cursor-pointer"
        >
          <Pin className="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onClick={() => onRemove(item.id)}
          title="Xóa"
          className="w-7 h-7 flex items-center justify-center rounded-md text-[#64748B] hover:text-[#DC2626] hover:bg-[#FEECEE] transition-colors cursor-pointer"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};
