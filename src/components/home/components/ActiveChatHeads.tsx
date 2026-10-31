import React from "react";
import { MoreHorizontal, Plus } from "lucide-react";
import { useConversationStore } from "../../../stores/conversationStore";
import {
  useMessengerStore,
  selectIsMessengerConnected,
} from "../../../stores/messengerStore";
import { useNavigationStore } from "../../../stores/navigationStore";
import { messengerService } from "../../../services/messenger";
import { CommonWidget } from "../../common";
import { Conversation } from "../../../types/conversation";
import { ActiveChatHeadItem } from "./ActiveChatHeadItem";

export const ActiveChatHeads: React.FC = () => {
  const conversations = useConversationStore((s) => s.conversations);
  const removeConversation = useConversationStore((s) => s.removeConversation);
  const togglePinConversation = useConversationStore(
    (s) => s.togglePinConversation,
  );
  const markConversationRead = useConversationStore(
    (s) => s.markConversationRead,
  );
  const isMessengerConnected = useMessengerStore(selectIsMessengerConnected);
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);

  const pinnedCount = conversations.filter((c) => c.isPinned).length;

  const handleOpenConversation = (item: Conversation) => {
    markConversationRead(item.id);
    messengerService.openThread(
      item.threadId || item.id,
      item.accountIndex || 1,
    );
  };

  return (
    <CommonWidget
      className="flex-1 h-[380px] lg:h-full"
      title="Chat heads đang hoạt động"
      badgeCount={conversations.length}
      headerAction={
        <button
          type="button"
          className="w-7 h-7 flex items-center justify-center text-[#6B7483] hover:text-[#202733] hover:bg-[#EEF2F9] rounded-[6px] transition-colors cursor-pointer"
        >
          <MoreHorizontal className="w-5 h-5" />
        </button>
      }
      bodyClassName="overflow-y-auto custom-scrollbar divide-y divide-[#E2E7EF]"
      footerLeft={
        <span className="text-[11px] text-[#6B7483]">
          {isMessengerConnected
            ? `${pinnedCount}/6 cuộc trò chuyện đã ghim`
            : "Cần kết nối Messenger"}
        </span>
      }
      footerRight={
        <button
          type="button"
          onClick={() =>
            setActiveTab(isMessengerConnected ? "home" : "connections")
          }
          disabled={!isMessengerConnected}
          className="flex items-center gap-1.5 text-[12px] font-medium text-[#0759DB] hover:text-[#0866FF] disabled:opacity-40 disabled:cursor-not-allowed transition-colors cursor-pointer"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>Thêm hội thoại</span>
        </button>
      }
    >
      {conversations.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-8 text-[#6B7483] gap-2.5 my-auto">
          <span className="text-[13px] font-medium text-[#202733]">
            Chưa có chat head nào hoạt động
          </span>
          <span className="text-[11px] text-[#6B7483] text-center max-w-[280px]">
            {isMessengerConnected
              ? "Thêm cuộc trò chuyện từ Messenger để ghim bong bóng lên desktop."
              : "Hãy kết nối tài khoản Messenger để bắt đầu sử dụng chat heads."}
          </span>
          <button
            onClick={() => setActiveTab("connections")}
            className="text-[12px] text-[#0759DB] font-medium hover:underline mt-1 cursor-pointer"
          >
            {isMessengerConnected
              ? "+ Thêm từ Messenger"
              : "Kết nối Messenger ngay ↗"}
          </button>
        </div>
      ) : (
        conversations.map((item) => (
          <ActiveChatHeadItem
            key={item.id}
            item={item}
            onOpen={handleOpenConversation}
            onTogglePin={togglePinConversation}
            onRemove={removeConversation}
          />
        ))
      )}
    </CommonWidget>
  );
};
