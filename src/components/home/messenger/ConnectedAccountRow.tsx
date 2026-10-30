import React from "react";
import { MessengerAccount } from "../../../types/account";
import { AvatarCircle, SwitchButton } from "../../common";

interface ConnectedAccountRowProps {
  account: MessengerAccount;
  onOpenChat: (account: MessengerAccount) => void;
  onToggleBubble: (account: MessengerAccount) => void;
}

export const ConnectedAccountRow: React.FC<ConnectedAccountRowProps> = ({
  account,
  onToggleBubble,
}) => {
  return (
    <div className="flex items-center justify-between p-3 bg-[#F8FAFD] hover:bg-[#F3F7FD] rounded-[10px] border border-[#E2E7EF]/80 transition-all gap-3 flex-wrap sm:flex-nowrap">
      {/* Left: Avatar & Info */}
      <div className="flex items-center gap-3 min-w-0">
        <div className="relative shrink-0">
          <AvatarCircle
            size={36}
            text={account.avatarText}
            bg={account.color}
            textColor="#FFFFFF"
            className="font-bold text-[12px] shadow-xs ring-2 ring-white"
          />
          <span className="absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full bg-[#10B981] ring-2 ring-white" />
        </div>

        <div className="flex flex-col min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-[13px] font-semibold text-[#202733] truncate leading-tight">
              {account.name}
            </span>
            {account.unreadCount > 0 && (
              <span className="px-1.5 py-0.5 rounded-full bg-[#E81123] text-white text-[10px] font-bold shrink-0 leading-none">
                {account.unreadCount > 99 ? "99+" : account.unreadCount}
              </span>
            )}
          </div>
          <span className="text-[11px] text-[#6B7483] truncate leading-tight mt-0.5">
            {account.isBubbleVisible
              ? "Bong bóng đang bật"
              : "Bong bóng đang ẩn"}{" "}
            • {account.lastSynced || "Đồng bộ vừa xong"}
          </span>
        </div>
      </div>

      {/* Right: Per-Bubble Toggle & Open Chat Button */}
      <div className="flex items-center gap-3 shrink-0 self-end sm:self-center">
        <div className="flex items-center gap-1.5 pr-1 border-r border-[#E2E7EF]">
          <span className="text-[11px] text-[#6B7483] hidden md:inline">
            Bong bóng
          </span>
          <SwitchButton
            checked={account.isBubbleVisible}
            onChange={() => onToggleBubble(account)}
            aria-label={`Bật tắt bong bóng cho ${account.name}`}
          />
        </div>
      </div>
    </div>
  );
};
