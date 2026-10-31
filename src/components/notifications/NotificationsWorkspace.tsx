import React, { useState } from "react";
import { useConversationStore } from "../../stores/conversationStore";
import { messengerService } from "../../services/messenger";
import { TabHeader } from "../common";
import { Conversation } from "../../types/conversation";
import { NotificationsHeaderActions, NotificationListCard } from "./components";
import { ClearNotificationsDialog } from "./dialogs";

export const NotificationsWorkspace: React.FC = () => {
  const conversations = useConversationStore((s) => s.conversations);
  const setConversations = useConversationStore((s) => s.setConversations);
  const markConversationRead = useConversationStore((s) => s.markConversationRead);
  const removeConversation = useConversationStore((s) => s.removeConversation);

  const [isConfirmClearOpen, setIsConfirmClearOpen] = useState(false);

  const totalUnread = conversations.reduce((acc, c) => acc + (c.unreadCount || 0), 0);

  const handleMarkAllRead = () => {
    const updated = conversations.map((c) => ({ ...c, unreadCount: 0 }));
    setConversations(updated);
  };

  const handleClearAll = () => {
    setConversations([]);
  };

  const handleOpenNotification = (item: Conversation) => {
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
          <NotificationsHeaderActions
            hasNotifications={conversations.length > 0}
            onMarkAllRead={handleMarkAllRead}
            onOpenClearDialog={() => setIsConfirmClearOpen(true)}
          />
        }
      />

      {/* 2. Nội dung cuộn mượt mà luôn bung hết chiều rộng */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <NotificationListCard
          conversations={conversations}
          totalUnread={totalUnread}
          onOpenNotification={handleOpenNotification}
          onRemove={removeConversation}
        />
      </main>

      {/* Dialog xác nhận xóa lịch sử thông báo */}
      <ClearNotificationsDialog
        isOpen={isConfirmClearOpen}
        onClose={() => setIsConfirmClearOpen(false)}
        onConfirm={handleClearAll}
      />
    </div>
  );
};
