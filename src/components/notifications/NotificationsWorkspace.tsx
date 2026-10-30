import React from "react";
import { Bell, CheckCheck, Trash2, ArrowUpRight, MessageCircle } from "lucide-react";
import { useConversationStore } from "../../stores/conversationStore";
import { messengerService } from "../../services/messenger";
import { AvatarCircle, TabHeader } from "../common";
import { getAccountThemeColor } from "../../constants/accountColors";

export const NotificationsWorkspace: React.FC = () => {
  const conversations = useConversationStore((s) => s.conversations);
  const setConversations = useConversationStore((s) => s.setConversations);
  const markConversationRead = useConversationStore((s) => s.markConversationRead);
  const removeConversation = useConversationStore((s) => s.removeConversation);

  const totalUnread = conversations.reduce((acc, c) => acc + (c.unreadCount || 0), 0);

  const handleMarkAllRead = () => {
    const updated = conversations.map((c) => ({ ...c, unreadCount: 0 }));
    setConversations(updated);
  };

  const handleClearAll = () => {
    setConversations([]);
  };

  const handleOpenNotification = (item: (typeof conversations)[0]) => {
    markConversationRead(item.id);
    messengerService.openThread(item.threadId || item.id, item.accountIndex || 1);
  };

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-[#F4F7FC] overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="TRUNG TÂM HOẠT ĐỘNG"
        title="Thông báo"
        description="Nhật ký tin nhắn và thông báo từ các tài khoản Messenger đã kết nối."
        rightSlot={
          <div className="flex items-center gap-2">
            {conversations.length > 0 && (
              <>
                <button
                  type="button"
                  onClick={handleMarkAllRead}
                  className="px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-lg text-[12px] font-medium text-[#1E293B] hover:bg-[#F8FAFC] flex items-center gap-1.5 transition-colors cursor-pointer shadow-xs"
                >
                  <CheckCheck className="w-3.5 h-3.5 text-[#0866FF]" />
                  <span>Đọc tất cả</span>
                </button>
                <button
                  type="button"
                  onClick={handleClearAll}
                  className="px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-lg text-[12px] font-medium text-[#DC2626] hover:bg-[#FEECEE] flex items-center gap-1.5 transition-colors cursor-pointer shadow-xs"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                  <span>Xóa lịch sử</span>
                </button>
              </>
            )}
          </div>
        }
      />

      {/* 2. Nội dung cuộn mượt mà luôn bung hết chiều rộng */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <div className="max-w-[880px] mx-auto w-full flex flex-col gap-6">
          {/* Notifications List Card */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-3">
          <div className="flex items-center justify-between pb-2 border-b border-[#E2E7EF]/60">
            <div className="flex items-center gap-2">
              <Bell className="w-4 h-4 text-[#64748B]" />
              <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
                Thông báo gần đây
              </h2>
            </div>
            {totalUnread > 0 && (
              <span className="px-2.5 py-0.5 rounded-full bg-[#E81123] text-white text-[11px] font-bold">
                {totalUnread} chưa đọc
              </span>
            )}
          </div>

          {conversations.length === 0 ? (
            <div className="py-12 flex flex-col items-center justify-center text-[#64748B] gap-2">
              <MessageCircle className="w-8 h-8 text-[#CBD5E1]" />
              <span className="text-[13px] font-medium text-[#1E293B]">Không có thông báo nào</span>
              <span className="text-[11px]">
                Khi có tin nhắn mới gửi đến Messenger, thông báo sẽ hiển thị ở đây.
              </span>
            </div>
          ) : (
            <div className="divide-y divide-[#E2E7EF]">
              {conversations.map((item) => {
                const color = getAccountThemeColor(item.accountIndex || 1);
                const isUnread = (item.unreadCount || 0) > 0;
                return (
                  <div
                    key={item.id}
                    onClick={() => handleOpenNotification(item)}
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
                          <span className={`text-[13px] truncate ${isUnread ? "font-bold text-[#0F172A]" : "font-medium text-[#334155]"}`}>
                            {item.name}
                          </span>
                          <span className="text-[10px] px-1.5 py-0.2 rounded text-white font-medium" style={{ backgroundColor: color.hex }}>
                            M{item.accountIndex || 1}
                          </span>
                        </div>
                        <p className={`text-[12px] truncate m-0 ${isUnread ? "text-[#1E293B] font-medium" : "text-[#64748B]"}`}>
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
                          removeConversation(item.id);
                        }}
                        title="Xóa thông báo"
                        className="w-7 h-7 flex items-center justify-center rounded-md text-[#94A3B8] hover:text-[#DC2626] hover:bg-[#FEECEE] transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                      <ArrowUpRight className="w-4 h-4 text-[#94A3B8] group-hover:text-[#0866FF]" />
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
        </div>
      </main>
    </div>
  );
};
