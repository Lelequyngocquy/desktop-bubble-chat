import React from "react";
import { Pin, ArrowUpRight } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Conversation } from "../../../types/conversation";
import { getAccountThemeColor } from "../../../constants/design";

interface PinnedChatItemProps {
  item: Conversation;
  onOpenThread: (item: Conversation) => void;
  onUnpin: (id: string) => void;
}

export const PinnedChatItem: React.FC<PinnedChatItemProps> = ({
  item,
  onOpenThread,
  onUnpin,
}) => {
  const { t } = useTranslation("dashboard");
  const color = getAccountThemeColor(item.accountIndex || 1);
  return (
    <div className="p-3.5 border border-border rounded-card flex items-center justify-between gap-3 hover:border-border-strong transition-all bg-background-subtle/50">
      <div className="flex items-center gap-3 min-w-0">
        <span
          className="w-2.5 h-2.5 rounded-pill shrink-0"
          style={{ backgroundColor: color.hex }}
          title={`Messenger ${item.accountIndex || 1}`}
        />
        <div className="min-w-0 flex flex-col">
          <span className="text-body font-semibold text-content-primary truncate">
            {item.name}
          </span>
          <span className="text-caption text-content-secondary truncate mt-0.5">
            {item.lastMessage || t("chatheads_page.no_messages")}
          </span>
        </div>
      </div>
      <div className="flex items-center gap-1.5 shrink-0">
        <button
          type="button"
          onClick={() => onOpenThread(item)}
          title={t("chatheads_page.open_chat")}
          className="w-8 h-8 flex items-center justify-center rounded-panel text-primary hover:bg-primary-subtle transition-colors cursor-pointer"
        >
          <ArrowUpRight className="w-4 h-4" />
        </button>
        <button
          type="button"
          onClick={() => onUnpin(item.id)}
          title={t("chatheads_page.unpin")}
          className="w-8 h-8 flex items-center justify-center rounded-panel text-primary hover:bg-status-errorBg hover:text-status-error transition-colors cursor-pointer"
        >
          <Pin className="w-4 h-4 fill-current -rotate-45" />
        </button>
      </div>
    </div>
  );
};

export default PinnedChatItem;
