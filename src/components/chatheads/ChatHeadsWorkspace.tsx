import React from "react";
import { Pin, MessageSquare, Trash2, ArrowUpRight, Users } from "lucide-react";
import { useConversationStore } from "../../stores/conversationStore";
import { useMessengerStore, selectIsMessengerConnected } from "../../stores/messengerStore";
import { useNavigationStore } from "../../stores/navigationStore";
import { messengerService } from "../../services/messenger";
import { AvatarCircle, TabHeader } from "../common";
import { getAccountThemeColor } from "../../constants/accountColors";

export const ChatHeadsWorkspace: React.FC = () => {
  const conversations = useConversationStore((s) => s.conversations);
  const removeConversation = useConversationStore((s) => s.removeConversation);
  const togglePinConversation = useConversationStore((s) => s.togglePinConversation);
  const markConversationRead = useConversationStore((s) => s.markConversationRead);
  const isMessengerConnected = useMessengerStore(selectIsMessengerConnected);
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);

  const pinnedList = conversations.filter((c) => c.isPinned);
  const recentList = conversations.filter((c) => !c.isPinned);

  const handleOpenThread = (item: (typeof conversations)[0]) => {
    markConversationRead(item.id);
    messengerService.openThread(item.threadId || item.id, item.accountIndex || 1);
  };

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-[#F4F7FC] overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="CUỘC TRÒ CHUYỆN NỔI"
        title="Quản lý Chat Heads"
        description="Cấu hình các cuộc trò chuyện nổi ưu tiên và bong bóng chat trên màn hình desktop."
        rightSlot={
          <span className="px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-full text-[12px] font-semibold text-[#0759DB] shadow-xs">
            {pinnedList.length}/6 đã ghim
          </span>
        }
      />

      {/* 2. Nội dung cuộn mượt mà luôn bung hết chiều rộng */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {/* Section 1: Pinned Chat Heads */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Pin className="w-4 h-4 text-[#0866FF] fill-current -rotate-45" />
              <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
                Cuộc trò chuyện đã ghim ({pinnedList.length}/6)
              </h2>
            </div>
            <span className="text-[11px] text-[#64748B]">Luôn hiển thị đầu tiên</span>
          </div>

          {pinnedList.length === 0 ? (
            <div className="py-8 text-center text-[#64748B] text-[12px]">
              Chưa có cuộc trò chuyện nào được ghim. Nhấn vào biểu tượng ghim ở danh sách bên dưới để cố định tối đa 6 người liên lạc.
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {pinnedList.map((item) => {
                const color = getAccountThemeColor(item.accountIndex || 1);
                return (
                  <div
                    key={item.id}
                    className="p-3.5 border border-[#E2E7EF] rounded-xl flex items-center justify-between gap-3 hover:border-[#CBD5E1] transition-all bg-[#F8FAFC]/50"
                  >
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
                        onClick={() => handleOpenThread(item)}
                        title="Mở chat"
                        className="w-8 h-8 flex items-center justify-center rounded-lg text-[#0866FF] hover:bg-[#EAF2FF] transition-colors cursor-pointer"
                      >
                        <ArrowUpRight className="w-4 h-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => togglePinConversation(item.id)}
                        title="Bỏ ghim"
                        className="w-8 h-8 flex items-center justify-center rounded-lg text-[#0866FF] hover:bg-[#FEECEE] hover:text-[#DC2626] transition-colors cursor-pointer"
                      >
                        <Pin className="w-4 h-4 fill-current -rotate-45" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Section 2: Recent Conversations */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <MessageSquare className="w-4 h-4 text-[#64748B]" />
              <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
                Hội thoại gần đây ({recentList.length})
              </h2>
            </div>
            {!isMessengerConnected && (
              <button
                type="button"
                onClick={() => setActiveTab("connections")}
                className="text-[12px] font-medium text-[#0759DB] hover:underline"
              >
                Kết nối Messenger để đồng bộ
              </button>
            )}
          </div>

          {recentList.length === 0 ? (
            <div className="py-8 text-center text-[#64748B] text-[12px]">
              Chưa có hội thoại nào gần đây.
            </div>
          ) : (
            <div className="divide-y divide-[#E2E7EF]">
              {recentList.map((item) => (
                <div
                  key={item.id}
                  className="py-3 flex items-center justify-between gap-3 hover:bg-[#F8FAFC] px-2 rounded-lg transition-colors"
                >
                  <div
                    onClick={() => handleOpenThread(item)}
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
                      onClick={() => togglePinConversation(item.id)}
                      disabled={pinnedList.length >= 6}
                      title={pinnedList.length >= 6 ? "Đã đạt tối đa 6 ghim" : "Ghim"}
                      className="w-7 h-7 flex items-center justify-center rounded-md text-[#64748B] hover:text-[#0866FF] hover:bg-[#EAF2FF] disabled:opacity-30 disabled:cursor-not-allowed transition-colors cursor-pointer"
                    >
                      <Pin className="w-3.5 h-3.5" />
                    </button>
                    <button
                      type="button"
                      onClick={() => removeConversation(item.id)}
                      title="Xóa"
                      className="w-7 h-7 flex items-center justify-center rounded-md text-[#64748B] hover:text-[#DC2626] hover:bg-[#FEECEE] transition-colors cursor-pointer"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </main>
    </div>
  );
};
