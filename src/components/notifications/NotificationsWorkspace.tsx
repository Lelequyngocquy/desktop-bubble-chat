import React, { useMemo, useState } from "react";
import { AlertCircle, ArrowRight } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useConversationStore } from "../../stores/conversationStore";
import { useNavigationStore } from "../../stores/navigationStore";
import { messengerService, useMessengerStore } from "../../platforms/messenger";
import { TabHeader } from "../common";
import { Conversation } from "../../types/conversation";
import { normalizeThreadId } from "../../utils/threadValidation";
import {
  NotificationsHeaderActions,
  NotificationListCard,
  PinnedChatHeadsCard,
} from "./components";
import { ClearNotificationsDialog } from "./dialogs";

export const NotificationsWorkspace: React.FC = () => {
  const { t } = useTranslation("dashboard");
  const conversations = useConversationStore((s) => s.conversations);
  const { setConversations, markConversationRead, removeConversation, togglePinConversation } = useConversationStore.getState();
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);
  const [isConfirmClearOpen, setIsConfirmClearOpen] = useState(false);
  const [disconnectedNotice, setDisconnectedNotice] = useState<string | null>(null);
  const totalUnread = useMemo(
    () => conversations.reduce((acc, c) => acc + (c.unreadCount || 0), 0),
    [conversations],
  );
  const pinnedList = useMemo(() => conversations.filter((c) => c.isPinned), [conversations]);
  const handleMarkAllRead = () => {
    const updated = conversations.map((c) => ({ ...c, unreadCount: 0 }));
    setConversations(updated);
  };
  const handleClearAll = () => {
    setConversations([]);
  };
  const handleOpenNotification = (item: Conversation) => {
    const accountIdx = item.accountIndex || 1;
    const paddedIndex = accountIdx.toString().padStart(2, "0");
    const accountId = `messenger-${paddedIndex}`;
    const targetAccount = useMessengerStore.getState().accounts.find((a) => a.id === accountId);
    if (!targetAccount || !targetAccount.isConnected) {
      setDisconnectedNotice(
        t("notifications_page.account_disconnected_warning", {
          account: targetAccount?.name || `Messenger ${paddedIndex}`,
        }),
      );
      return;
    }
    setDisconnectedNotice(null);
    markConversationRead(item.id);
    const validThreadId = normalizeThreadId(item.threadId);
    if (validThreadId) {
      messengerService.openThread(validThreadId, accountIdx).catch((err) => {
        console.warn("[NotificationsWorkspace] Failed to open thread:", err);
      });
    } else {
      messengerService.openChat(accountIdx).catch((err) => {
        console.warn("[NotificationsWorkspace] Failed to open chat:", err);
      });
    }
  };
  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      <TabHeader
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
        {disconnectedNotice && (
          <div
            role="alert"
            className="flex items-center justify-between gap-3 px-4 py-3 rounded-card-sm bg-status-warningBg border border-status-warningBorder text-small"
          >
            <div className="flex items-center gap-2.5 text-status-warning font-medium min-w-0">
              <AlertCircle className="w-4 h-4 shrink-0" />
              <span className="truncate">{disconnectedNotice}</span>
            </div>
            <button
              type="button"
              onClick={() => setActiveTab("connections")}
              className="flex items-center gap-1 text-caption font-semibold text-primary hover:underline shrink-0 cursor-pointer"
            >
              <span>{t("navigation.connections")}</span>
              <ArrowRight className="w-3.5 h-3.5" />
            </button>
          </div>
        )}
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
