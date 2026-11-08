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
  const accounts = useMessengerStore((s) => s.accounts);
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);
  const pinnedCount = conversations.filter((c) => c.isPinned).length;
  const handleOpenConversation = (item: Conversation) => {
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
    <CommonWidget
      className="flex-1 h-side-col lg:h-full"
      title={t("chatheads.title")}
      badgeCount={conversations.length}
      headerAction={
        <button
          type="button"
          disabled
          title={t("chatheads.feature_not_available")}
          className="w-avatar-sm h-avatar-sm flex items-center justify-center text-content-secondary opacity-50 cursor-not-allowed rounded-control transition-colors"
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
          disabled
          title={t("chatheads.feature_not_available")}
          className="flex items-center gap-1.5 text-small font-medium text-primary-foreground opacity-40 cursor-not-allowed transition-colors"
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
          {isMessengerConnected ? (
            <button
              type="button"
              disabled
              title={t("chatheads.feature_not_available")}
              className="text-small text-content-secondary font-medium opacity-50 cursor-not-allowed mt-1"
            >
              {t("chatheads.empty_btn_add")}
            </button>
          ) : (
            <button
              type="button"
              onClick={() => setActiveTab("connections")}
              className="text-small text-primary-foreground font-medium hover:underline mt-1 cursor-pointer"
            >
              {t("chatheads.empty_btn_connect")}
            </button>
          )}
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