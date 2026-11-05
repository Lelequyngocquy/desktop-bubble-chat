import React from "react";
import { useTranslation } from "react-i18next";
import { useConversationStore } from "../../stores/conversationStore";
import { useNavigationStore } from "../../stores/navigationStore";
import {
  useMessengerStore,
  selectIsMessengerConnected,
  messengerService,
} from "../../platforms/messenger";
import { TabHeader } from "../common";
import { Conversation } from "../../types/conversation";
import { PinnedChatHeadsCard, RecentConversationsCard } from "./components";

export const ChatHeadsWorkspace: React.FC = () => {
  const { t } = useTranslation("dashboard");
  const conversations = useConversationStore((s) => s.conversations);
  const removeConversation = useConversationStore((s) => s.removeConversation);
  const togglePinConversation = useConversationStore((s) => s.togglePinConversation);
  const markConversationRead = useConversationStore((s) => s.markConversationRead);
  const isMessengerConnected = useMessengerStore(selectIsMessengerConnected);
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);
  const pinnedList = conversations.filter((c) => c.isPinned);
  const recentList = conversations.filter((c) => !c.isPinned);
  const handleOpenThread = (item: Conversation) => {
    markConversationRead(item.id);
    messengerService.openThread(item.threadId || item.id, item.accountIndex || 1);
  };

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      <TabHeader
        eyebrow={t("workspaces.chatheads.eyebrow")}
        title={t("workspaces.chatheads.title")}
        description={t("workspaces.chatheads.description")}
        rightSlot={
          <span className="px-3 py-1.5 bg-background-surface border border-border rounded-pill text-small font-semibold text-primary-foreground shadow-xs">
            {t("workspaces.chatheads.pinned_badge", { count: pinnedList.length })}
          </span>
        }
      />
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        <PinnedChatHeadsCard
          pinnedList={pinnedList}
          onOpenThread={handleOpenThread}
          onUnpin={togglePinConversation}
        />
        <RecentConversationsCard
          recentList={recentList}
          canPin={pinnedList.length < 6}
          isMessengerConnected={isMessengerConnected}
          onConnectMessenger={() => setActiveTab("connections")}
          onOpenThread={handleOpenThread}
          onPin={togglePinConversation}
          onRemove={removeConversation}
        />
      </main>
    </div>
  );
};

export default ChatHeadsWorkspace;