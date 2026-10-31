import React from "react";
import { Pin, ArrowUpRight, Users } from "lucide-react";
import { Conversation } from "../../../types/conversation";
import { AvatarCircle } from "../../common";
import { getAccountThemeColor } from "../../../constants/accountColors";

interface PinnedChatItemProps {
  item: Conversation;
  onOpenThread: (item: Conversation) => void;
  onUnpin: (id: string) => void;
}

export const PinnedChatItem: React.FC<PinnedChatItemProps> = ({
  item,
  onOpenThread,
  onUnpin,
}) => {
  const color = getAccountThemeColor(item.accountIndex || 1);

  return (
    <div className="p-3.5 border border-[#E2E7EF] rounded-xl flex items-center justify-between gap-3 hover:border-[#CBD5E1] transition-all bg-[#F8FAFC]/50">
      <div className="flex items-center gap-3 min-w-0">
        {item.avatarUrl ? (
          <img
            src={item.avatarUrl}
            alt={item.name}
            className="w-10 h-10 rounded-full object-cover shrink-0 border border-[#CBD5E1]"
          />
        ) : (
          <AvatarCircle
            size={40}
            icon={item.isGroup ? <Users className="w-5 h-5 text-[#7762AC]" /> : undefined}
            text={!item.isGroup ? item.avatarText || item.name.slice(0, 2).toUpperCase() : undefined}
            bg={item.isGroup ? "#F2EEFF" : "#EAF2FF"}
            textColor={item.isGroup ? "#7762AC" : "#0759DB"}
          />
        )}
        <div className="min-w-0 flex flex-col">
          <span className="text-[13px] font-semibold text-[#1E293B] truncate">
            {item.name}
          </span>
          <div className="flex items-center gap-1.5 mt-0.5">
            <span
              className="w-2 h-2 rounded-full shrink-0"
              style={{ backgroundColor: color.hex }}
            />
            <span className="text-[11px] text-[#64748B] truncate">
              {item.lastMessage || "Chưa có tin nhắn"}
            </span>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-1.5 shrink-0">
        <button
          type="button"
          onClick={() => onOpenThread(item)}
          title="Mở chat"
          className="w-8 h-8 flex items-center justify-center rounded-lg text-[#0866FF] hover:bg-[#EAF2FF] transition-colors cursor-pointer"
        >
          <ArrowUpRight className="w-4 h-4" />
        </button>
        <button
          type="button"
          onClick={() => onUnpin(item.id)}
          title="Bỏ ghim"
          className="w-8 h-8 flex items-center justify-center rounded-lg text-[#0866FF] hover:bg-[#FEECEE] hover:text-[#DC2626] transition-colors cursor-pointer"
        >
          <Pin className="w-4 h-4 fill-current -rotate-45" />
        </button>
      </div>
    </div>
  );
};
