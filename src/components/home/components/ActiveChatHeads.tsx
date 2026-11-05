import React from "react";
import { MoreHorizontal, Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useConversationStore } from "../../../stores/conversationStore";
import { useNavigationStore } from "../../../stores/navigationStore";
import {
  useMessengerStore,
  selectIsMessengerConnected,
  messengerService,
} from "../../../platforms/messenger";
import { CommonWidget } from "../../common";
import { Conversation } from "../../../types/conversation";
import { ActiveChatHeadItem } from "./ActiveChatHeadItem";

export const ActiveChatHeads: React.FC = () => {
  const { t } = useTranslation("dashboard");
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
      className="flex-1 h-side-col lg:h-full"
      title={t("chatheads.title")}
      badgeCount={conversations.length}
      headerAction={
        <button
          type="button"
          className="w-avatar-sm h-avatar-sm flex items-center justify-center text-content-secondary hover:text-content-primary hover:bg-background-app rounded-control transition-colors cursor-pointer"
        >
          <MoreHorizontal className="w-5 h-5" />
        </button>
      }
      bodyClassName="overflow-y-auto custom-scrollbar divide-y divide-border"
      footerLeft={
        <span className="text-caption text-content-secondary">
          {isMessengerConnected
            ? t("chatheads.pinned_ratio", { pinned: pinnedCount })
            : t("chatheads.need_connect")}
        </span>
      }
      footerRight={
        <button
          type="button"
          onClick={() =>
            setActiveTab(isMessengerConnected ? "home" : "connections")
          }
          disabled={!isMessengerConnected}
          className="flex items-center gap-1.5 text-small font-medium text-primary-foreground hover:text-primary disabled:opacity-40 disabled:cursor-not-allowed transition-colors cursor-pointer"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>{t("chatheads.add_conversation")}</span>
        </button>
      }
    >
      {conversations.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-8 text-content-secondary gap-2.5 my-auto">
          <span className="text-body font-medium text-content-primary">
            {t("chatheads.empty_title")}
          </span>
          <span className="text-caption text-content-secondary text-center max-w-[280px]">
            {isMessengerConnected
              ? t("chatheads.empty_desc_connected")
              : t("chatheads.empty_desc_disconnected")}
          </span>
          <button
            onClick={() => setActiveTab("connections")}
            className="text-small text-primary-foreground font-medium hover:underline mt-1 cursor-pointer"
          >
            {isMessengerConnected
              ? t("chatheads.empty_btn_add")
              : t("chatheads.empty_btn_connect")}
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

export default ActiveChatHeads;