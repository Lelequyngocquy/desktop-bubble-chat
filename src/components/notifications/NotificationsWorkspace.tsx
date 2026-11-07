import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useConversationStore } from "../../stores/conversationStore";
import { messengerService, useMessengerStore } from "../../platforms/messenger";
import { TabHeader } from "../common";
import { Conversation } from "../../types/conversation";
import {
  NotificationsHeaderActions,
  NotificationListCard,
  PinnedChatHeadsCard,
} from "./components";
import { ClearNotificationsDialog } from "./dialogs";

export const NotificationsWorkspace: React.FC = () => {
  const { t } = useTranslation("dashboard");
  const accounts = useMessengerStore((s) => s.accounts);
  const conversations = useConversationStore((s) => s.conversations);
  const setConversations = useConversationStore((s) => s.setConversations);
  const markConversationRead = useConversationStore((s) => s.markConversationRead);
  const removeConversation = useConversationStore((s) => s.removeConversation);
  const togglePinConversation = useConversationStore((s) => s.togglePinConversation);
  const [isConfirmClearOpen, setIsConfirmClearOpen] = useState(false);
  const totalUnread = conversations.reduce((acc, c) => acc + (c.unreadCount || 0), 0);
  const pinnedList = conversations.filter((c) => c.isPinned);
  const handleMarkAllRead = () => {
    const updated = conversations.map((c) => ({ ...c, unreadCount: 0 }));
    setConversations(updated);
  };
  const handleClearAll = () => {
    setConversations([]);
  };
  const handleOpenNotification = (item: Conversation) => {
    const accountIdx = item.accountIndex || 1;
    const accountId = `messenger-${accountIdx.toString().padStart(2, "0")}`;
    const targetAccount = accounts.find((a) => a.id === accountId);
    if (!targetAccount || !targetAccount.isConnected) {
      removeConversation(item.id);
      return;
    }
    markConversationRead(item.id);
    messengerService.openThread(item.threadId || item.id, accountIdx).catch(() => {});
  };
  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      <TabHeader
        eyebrow={t("workspaces.notifications.eyebrow")}
        title={t("workspaces.notifications.title")}
        description={t("workspaces.notifications.description")}
        rightSlot={
          <NotificationsHeaderActions
            hasNotifications={conversations.length > 0}
            onMarkAllRead={handleMarkAllRead}
            onOpenClearDialog={() => setIsConfirmClearOpen(true)}
          />
        }
      />
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {pinnedList.length > 0 && (
          <PinnedChatHeadsCard
            pinnedList={pinnedList}
            onOpenThread={handleOpenNotification}
            onUnpin={togglePinConversation}
          />
        )}
        <NotificationListCard
          conversations={conversations}
          totalUnread={totalUnread}
          onOpenNotification={handleOpenNotification}
          onRemove={removeConversation}
          onTogglePin={togglePinConversation}
        />
      </main>
      <ClearNotificationsDialog
        isOpen={isConfirmClearOpen}
        onClose={() => setIsConfirmClearOpen(false)}
        onConfirm={handleClearAll}
      />
    </div>
  );
};

export default NotificationsWorkspace;
