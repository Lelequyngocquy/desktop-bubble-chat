import React from "react";
import { Bell, MessageCircle } from "lucide-react";
import { Conversation } from "../../../types/conversation";
import { NotificationItemRow } from "./NotificationItemRow";
import { CommonLargeCard } from "../../common/index";

interface NotificationListCardProps {
  conversations: Conversation[];
  totalUnread: number;
  onOpenNotification: (item: Conversation) => void;
  onRemove: (id: string) => void;
}

export const NotificationListCard: React.FC<NotificationListCardProps> = ({
  conversations,
  totalUnread,
  onOpenNotification,
  onRemove,
}) => {
  return (
    <CommonLargeCard 
      icon={<Bell className="w-4 h-4 text-[#0866FF] fill-current" />}
      title="Thông báo gần đây"
      subTitle={
        totalUnread > 0 ? (
          <span className="px-2.5 py-0.5 rounded-full bg-[#E81123] text-white text-[11px] font-bold">
            {totalUnread} chưa đọc
          </span>
        ) : undefined
      }
      className="gap-3"
    >
      {conversations.length === 0 ? (
        <div className="py-12 flex flex-col items-center justify-center text-[#64748B] gap-2">
          <MessageCircle className="w-8 h-8 text-[#CBD5E1]" />
          <span className="text-[13px] font-medium text-[#1E293B]">
            Không có thông báo nào
          </span>
          <span className="text-[11px]">
            Khi có tin nhắn mới gửi đến Messenger, thông báo sẽ hiển thị ở đây.
          </span>
        </div>
      ) : (
        <div className="divide-y divide-[#E2E7EF]">
          {conversations.map((item) => (
            <NotificationItemRow
              key={item.id}
              item={item}
              onOpenNotification={onOpenNotification}
              onRemove={onRemove}
            />
          ))}
        </div>
      )}
    </CommonLargeCard>
  );
};
