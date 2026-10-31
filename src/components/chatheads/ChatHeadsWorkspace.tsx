import React from "react";
import { useConversationStore } from "../../stores/conversationStore";
import { useMessengerStore, selectIsMessengerConnected } from "../../stores/messengerStore";
import { useNavigationStore } from "../../stores/navigationStore";
import { messengerService } from "../../services/messenger";
import { TabHeader } from "../common";
import { Conversation } from "../../types/conversation";
import { PinnedChatHeadsCard, RecentConversationsCard } from "./components";

export const ChatHeadsWorkspace: React.FC = () => {
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
