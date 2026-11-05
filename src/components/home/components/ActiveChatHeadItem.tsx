import React from "react";
import { X, Pin } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Conversation } from "../../../types/conversation";
import { getAccountThemeColor } from "../../../constants/design";

interface ActiveChatHeadItemProps {
  item: Conversation;
  onOpen: (item: Conversation) => void;
  onTogglePin: (id: string) => void;
  onRemove: (id: string) => void;
}

export const ActiveChatHeadItem: React.FC<ActiveChatHeadItemProps> = ({
  item,
  onOpen,
  onTogglePin,
  onRemove,
}) => {
  const { t } = useTranslation("dashboard");
  const color = getAccountThemeColor(item.accountIndex || 1);

  return (
    <div
      onClick={() => onOpen(item)}
      className="h-[52px] flex items-center gap-3 py-1 px-2 -mx-2 rounded-panel hover:bg-background-subtle cursor-pointer group transition-colors"
    >
      <span
        className="w-2.5 h-2.5 rounded-pill shrink-0 ml-1"
        style={{ backgroundColor: color.hex }}
        title={`Messenger ${item.accountIndex || 1}`}
      />
      <div className="flex-1 min-w-0 flex flex-col gap-0.5">
        <div className="flex items-center gap-1.5">
          <span className="text-small font-semibold text-content-primary truncate">
            {item.name}
          </span>
          {item.isPinned && (
            <Pin className="w-3 h-3 text-primary fill-current shrink-0 -rotate-45" />
          )}
          {item.unreadCount > 0 && (
            <span className="h-4 min-w-4 px-1 rounded-pill bg-primary text-content-inverse text-badge font-bold flex items-center justify-center shrink-0 leading-none">
              {item.unreadCount}
            </span>
          )}
        </div>
        <p className="text-caption text-content-secondary truncate m-0 leading-tight">
          {item.lastMessage}
        </p>
      </div>
      <div className="flex items-center gap-1 shrink-0">
        <span className="text-caption text-content-muted mr-0.5">
          {item.timestamp}
        </span>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onTogglePin(item.id);
          }}
          title={item.isPinned ? t("chatheads.unpin") : t("chatheads.pin")}
          className={`w-6 h-6 flex items-center justify-center rounded-xs transition-all cursor-pointer ${
            item.isPinned
              ? "text-primary hover:bg-primary-subtle"
              : "opacity-0 group-hover:opacity-100 text-content-secondary hover:text-primary hover:bg-background-app"
          }`}
        >
          <Pin
            className={`w-3.5 h-3.5 ${item.isPinned ? "fill-current -rotate-45" : ""}`}
          />
        </button>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onRemove(item.id);
          }}
          title={t("chatheads.hide")}
          className="opacity-0 group-hover:opacity-100 w-6 h-6 flex items-center justify-center text-content-secondary hover:text-status-error hover:bg-status-errorBg rounded-xs transition-all cursor-pointer"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};

export default ActiveChatHeadItem;