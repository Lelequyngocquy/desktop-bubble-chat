import React from "react";
import { MessageSquare } from "lucide-react";
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
  return (
    <CommonLargeCard
      icon={<MessageSquare className="w-4 h-4 text-[#0866FF] fill-current" />}
      title={`Hội thoại gần đây (${recentList.length})`}
      linkTitle={
        !isMessengerConnected ? (
          <button
            type="button"
            onClick={onConnectMessenger}
            className="text-[12px] font-medium text-[#0759DB] hover:underline cursor-pointer"
          >
            Kết nối Messenger để đồng bộ
          </button>
        ) : undefined
      }
    >
      {recentList.length === 0 ? (
        <div className="py-8 text-center text-[#64748B] text-[12px]">
          Chưa có hội thoại nào gần đây.
        </div>
      ) : (
        <div className="divide-y divide-[#E2E7EF]">
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
