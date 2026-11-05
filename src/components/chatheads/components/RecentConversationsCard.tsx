import React from "react";
import { MessageSquare } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Conversation } from "../../../types/conversation";
import { RecentChatItem } from "./RecentChatItem";
import { CommonLargeCard } from "../../common/index";

interface RecentConversationsCardProps {
  recentList: Conversation[];
  canPin: boolean;
  isMessengerConnected: boolean;
  onConnectMessenger: () => void;
  onOpenThread: (item: Conversation) => void;
  onPin: (id: string) => void;
  onRemove: (id: string) => void;
}

export const RecentConversationsCard: React.FC<
  RecentConversationsCardProps
> = ({
  recentList,
  canPin,
  isMessengerConnected,
  onConnectMessenger,
  onOpenThread,
  onPin,
  onRemove,
}) => {
  const { t } = useTranslation("dashboard");

  return (
    <CommonLargeCard
      icon={<MessageSquare className="w-4 h-4 text-primary fill-current" />}
      title={t("chatheads_page.recent_title", { count: recentList.length })}
      linkTitle={
        !isMessengerConnected ? (
          <button
            type="button"
            onClick={onConnectMessenger}
            className="text-small font-medium text-primary-foreground hover:underline cursor-pointer"
          >
            {t("chatheads_page.connect_to_sync")}
          </button>
        ) : undefined
      }
    >
      {recentList.length === 0 ? (
        <div className="py-8 text-center text-content-secondary text-small">
          {t("chatheads_page.recent_empty")}
        </div>
      ) : (
        <div className="divide-y divide-border">
          {recentList.map((item) => (
            <RecentChatItem
              key={item.id}
              item={item}
              canPin={canPin}
              onOpenThread={onOpenThread}
              onPin={onPin}
              onRemove={onRemove}
            />
          ))}
        </div>
      )}
    </CommonLargeCard>
  );
};

export default RecentConversationsCard;