import React from "react";
import { Users, MoreHorizontal, X, Plus } from "lucide-react";
import { useAppStore } from "../../stores/appStore";
import { AvatarCircle } from "../common";

export const ActiveChatHeads: React.FC = () => {
  const { conversations, removeConversation } = useAppStore();

  return (
    <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card flex flex-col justify-between flex-1 min-h-[352px] select-none">
      {/* Top Heading (Figma Node #3:1250) */}
      <div className="p-[20px_22px_12px] flex flex-col gap-1.5 border-b border-[#E2E7EF]/50">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h2 className="text-[17px] font-semibold text-[#202733] m-0">
              Chat heads đang hoạt động
            </h2>
            <span className="px-[7px] py-[1px] bg-[#EAF2FF] text-[#0759DB] text-[11px] font-semibold rounded-[6px]">
              {conversations.length}
            </span>
          </div>

          <button
            type="button"
            className="w-7 h-7 flex items-center justify-center text-[#6B7483] hover:text-[#202733] hover:bg-[#EEF2F9] rounded-[6px] transition-colors"
          >
            <MoreHorizontal className="w-5 h-5" />
          </button>
        </div>
        <p className="text-[12px] text-[#6B7483] m-0">
          Các cuộc trò chuyện đã ghim trên desktop · Messenger
        </p>
      </div>

      {/* Pinned Conversation List (Figma Node #3:1258) */}
      <div className="px-[22px] flex flex-col flex-1 divide-y divide-[#E2E7EF]">
        {conversations.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-10 text-[#6B7483] gap-2">
            <span className="text-[13px]">Chưa có cuộc trò chuyện nào được ghim</span>
            <button className="text-[12px] text-[#0759DB] font-medium hover:underline">
              + Ghim cuộc trò chuyện
            </button>
          </div>
        ) : (
          conversations.map((item) => (
            <div
              key={item.id}
              className="h-[67px] flex items-center gap-3 py-2 group"
            >
              {/* Avatar / Portrait */}
              <AvatarCircle
                size={40}
                icon={item.isGroup ? <Users className="w-5 h-5 text-[#7762AC]" /> : undefined}
                text={!item.isGroup ? item.avatarText || item.name.slice(0, 2).toUpperCase() : undefined}
                bg={item.isGroup ? "#F2EEFF" : "#EAF2FF"}
                textColor={item.isGroup ? "#7762AC" : "#0759DB"}
                border={!item.isGroup}
              />

              {/* Conversation summary */}
              <div className="flex-1 min-w-0 flex flex-col gap-[3px]">
                <div className="flex items-center gap-2">
                  <span className="text-[13px] font-semibold text-[#202733] truncate">
                    {item.name}
                  </span>
                  {item.unreadCount > 0 && (
                    <span className="h-[18px] min-w-[18px] px-1.5 rounded-full bg-[#0866FF] text-white text-[10px] font-semibold flex items-center justify-center shrink-0">
                      {item.unreadCount}
                    </span>
                  )}
                </div>
                <p className="text-[12px] text-[#6B7483] truncate m-0">
                  {item.lastMessage}
                </p>
              </div>

              {/* Timestamp & Dismiss Action */}
              <div className="flex items-center gap-3 shrink-0">
                <span className="text-[11px] text-[#929AA7]">
                  {item.timestamp}
                </span>
                <button
                  type="button"
                  onClick={() => removeConversation(item.id)}
                  title="Ẩn chat head"
                  className="w-7 h-7 rounded-[6px] text-[#6B7483] hover:text-[#202733] hover:bg-[#EEF2F9] flex items-center justify-center transition-colors"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Footer Add Action (Figma Node #3:1294) */}
      <div className="h-14 px-[22px] flex items-center justify-between border-t border-[#E2E7EF]/50 bg-white rounded-b-[12px]">
        <button
          type="button"
          className="flex items-center gap-1.5 text-[#0759DB] hover:text-[#0866FF] text-[12px] font-medium transition-colors"
        >
          <Plus className="w-4 h-4" />
          <span>Thêm từ Messenger</span>
        </button>

        <span className="text-[11px] text-[#929AA7]">
          Dấu × chỉ ẩn chat head
        </span>
      </div>
    </div>
  );
};
