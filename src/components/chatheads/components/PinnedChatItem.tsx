import React from "react";
import { Pin, ArrowUpRight } from "lucide-react";
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
  const color = getAccountThemeColor(item.accountIndex || 1);

  return (
    <div className="p-3.5 border border-border rounded-card flex items-center justify-between gap-3 hover:border-border-strong transition-all bg-background-subtle/50">
      <div className="flex items-center gap-3 min-w-0">
        {/* Hủy bỏ avatar: thay bằng chấm tròn chỉ báo màu tài khoản Messenger */}
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
            {item.lastMessage || "Chưa có tin nhắn"}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-1.5 shrink-0">
        <button
          type="button"
          onClick={() => onOpenThread(item)}
          title="Mở chat"
          className="w-8 h-8 flex items-center justify-center rounded-panel text-primary hover:bg-primary-subtle transition-colors cursor-pointer"
        >
          <ArrowUpRight className="w-4 h-4" />
        </button>
        <button
          type="button"
          onClick={() => onUnpin(item.id)}
          title="Bỏ ghim"
          className="w-8 h-8 flex items-center justify-center rounded-panel text-primary hover:bg-status-errorBg hover:text-status-error transition-colors cursor-pointer"
        >
          <Pin className="w-4 h-4 fill-current -rotate-45" />
        </button>
      </div>
    </div>
  );
};

export default PinnedChatItem;
