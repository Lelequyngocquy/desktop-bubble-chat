import React from "react";
import { Trash2, ArrowUpRight, Pin } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Conversation } from "../../../types/conversation";
import { getAccountThemeColor } from "../../../constants/design";
import { formatMessageTime } from "../../../stores/conversationStore";

interface NotificationItemRowProps {
  item: Conversation;
  onOpenNotification: (item: Conversation) => void;
  onRemove: (id: string) => void;
  onTogglePin?: (id: string) => void;
}

export const NotificationItemRow: React.FC<NotificationItemRowProps> = ({
  item,
  onOpenNotification,
  onRemove,
  onTogglePin,
}) => {
  const { t } = useTranslation("dashboard");
  const color = getAccountThemeColor(item.accountIndex || 1);
  const isUnread = (item.unreadCount || 0) > 0;
  const formattedTime = item.timestampMs ? formatMessageTime(item.timestampMs, t) : item.timestamp;
  return (
    <div
      className={`py-3.5 px-3 flex items-center justify-between gap-3.5 rounded-card transition-all group ${
        isUnread
          ? "bg-primary-subtle hover:bg-primary-subtle/80"
          : "hover:bg-background-subtle"
      }`}
    >
      <button
        type="button"
        onClick={() => onOpenNotification(item)}
        className="flex items-center gap-3 min-w-0 flex-1 text-left cursor-pointer"
      >
        <span
          className="w-2.5 h-2.5 rounded-pill shrink-0"
          style={{ backgroundColor: color.hex }}
          title={`Messenger ${item.accountIndex || 1}`}
        />
        <span className="flex-1 min-w-0 flex flex-col gap-0.5">
          <span
            className={`text-body truncate block ${
              isUnread ? "font-bold text-content-primary" : "font-medium text-content-primary"
            }`}
          >
            {item.name}
          </span>
          <span
            className={`text-small truncate block ${
              isUnread ? "text-content-primary font-medium" : "text-content-secondary"
            }`}
          >
            {item.lastMessage}
          </span>
        </span>
      </button>
      <div className="flex items-center gap-2 shrink-0">
        <span className="text-caption text-content-muted">{formattedTime}</span>
        {onTogglePin && (
          <button
            type="button"
            onClick={() => onTogglePin(item.id)}
            title={item.isPinned ? t("chatheads_page.unpin") : t("chatheads_page.pin")}
            aria-label={item.isPinned ? t("chatheads_page.unpin") : t("chatheads_page.pin")}
            className={`w-7 h-7 flex items-center justify-center rounded-control transition-colors cursor-pointer ${
              item.isPinned
                ? "text-primary hover:bg-status-errorBg hover:text-status-error"
                : "text-content-muted hover:text-primary hover:bg-background-subtle"
            }`}
          >
            <Pin className={`w-3.5 h-3.5 ${item.isPinned ? "fill-current -rotate-45" : ""}`} />
          </button>
        )}
        <button
          type="button"
          onClick={() => onRemove(item.id)}
          title={t("notifications_page.delete_item")}
          aria-label={t("notifications_page.delete_item")}
          className="w-7 h-7 flex items-center justify-center rounded-control text-content-muted hover:text-status-error hover:bg-status-errorBg transition-colors cursor-pointer"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
        <ArrowUpRight className="w-4 h-4 text-content-muted group-hover:text-primary pointer-events-none" />
      </div>
    </div>
  );
};

export default NotificationItemRow;
