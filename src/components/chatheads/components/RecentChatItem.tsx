import React from "react";
import { Pin, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Conversation } from "../../../types/conversation";
import { getAccountThemeColor } from "../../../constants/design";

interface RecentChatItemProps {
  item: Conversation;
  canPin: boolean;
  onOpenThread: (item: Conversation) => void;
  onPin: (id: string) => void;
  onRemove: (id: string) => void;
}

export const RecentChatItem: React.FC<RecentChatItemProps> = ({
  item,
  canPin,
  onOpenThread,
  onPin,
  onRemove,
}) => {
  const { t } = useTranslation("dashboard");
  const color = getAccountThemeColor(item.accountIndex || 1);

  return (
    <div className="py-3 flex items-center justify-between gap-3 hover:bg-background-subtle px-2 rounded-panel transition-colors">
      <div
        onClick={() => onOpenThread(item)}
        className="flex items-center gap-3 min-w-0 flex-1 cursor-pointer"
      >
        <span
          className="w-2.5 h-2.5 rounded-pill shrink-0"
          style={{ backgroundColor: color.hex }}
          title={`Messenger ${item.accountIndex || 1}`}
        />
        <div className="min-w-0 flex flex-col">
          <span className="text-body font-medium text-content-primary truncate">
            {item.name}
          </span>
          <span className="text-caption text-content-secondary truncate">
            {item.lastMessage}
          </span>
        </div>
      </div>
      <div className="flex items-center gap-2 shrink-0">
        <span className="text-caption text-content-muted mr-1">{item.timestamp}</span>
        <button
          type="button"
          onClick={() => onPin(item.id)}
          disabled={!canPin}
          title={!canPin ? t("chatheads_page.max_pinned") : t("chatheads_page.pin")}
          className="w-7 h-7 flex items-center justify-center rounded-control text-content-secondary hover:text-primary hover:bg-primary-subtle disabled:opacity-30 disabled:cursor-not-allowed transition-colors cursor-pointer"
        >
          <Pin className="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onClick={() => onRemove(item.id)}
          title={t("chatheads_page.delete")}
          className="w-7 h-7 flex items-center justify-center rounded-control text-content-secondary hover:text-status-error hover:bg-status-errorBg transition-colors cursor-pointer"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};

export default RecentChatItem;