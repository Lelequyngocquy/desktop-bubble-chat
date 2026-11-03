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
      icon={<Bell className="w-4 h-4 text-primary fill-current" />}
      title="Thông báo gần đây"
      subTitle={
        totalUnread > 0 ? (
          <span className="px-2.5 py-0.5 rounded-pill bg-status-error text-content-inverse text-caption font-bold">
            {totalUnread} chưa đọc
          </span>
        ) : undefined
      }
      className="gap-3"
    >
      {conversations.length === 0 ? (
        <div className="py-12 flex flex-col items-center justify-center text-content-secondary gap-2">
          <MessageCircle className="w-8 h-8 text-border" />
          <span className="text-body font-medium text-content-primary">
            Không có thông báo nào
          </span>
          <span className="text-caption">
            Khi có tin nhắn mới gửi đến Messenger, thông báo sẽ hiển thị ở đây.
          </span>
        </div>
      ) : (
        <div className="divide-y divide-border">
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

export default NotificationListCard;
