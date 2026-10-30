import React, { useMemo } from "react";
import { ShieldCheck, LogIn } from "lucide-react";
import { messengerService } from "../../../services/messenger";
import {
  useMessengerStore,
  selectTotalUnreadCount,
  selectVisibleBubbleCount,
  selectAreAnyBubblesVisible,
} from "../../../stores/messengerStore";
import { useWindowStore } from "../../../stores/windowStore";
import { useNavigationStore } from "../../../stores/navigationStore";
import { MessengerAccount } from "../../../types/account";
import { CommonButton } from "../../common";
import { MessengerPlatformHeader } from "./MessengerPlatformHeader";
import { ConnectedAccountRow } from "./ConnectedAccountRow";
import { ConnectionMetricsColumn } from "./ConnectionMetricsColumn";

export const MessengerConnectionCard: React.FC = () => {
  const accounts = useMessengerStore((s) => s.accounts);
  const connectedAccounts = useMemo(
    () => accounts.filter((a) => a.isConnected),
    [accounts],
  );
  const totalUnreadCount = useMessengerStore(selectTotalUnreadCount);
  const visibleBubbleCount = useMessengerStore(selectVisibleBubbleCount);
  const areAnyBubblesVisible = useMessengerStore(selectAreAnyBubblesVisible);

  const toggleAccountBubbleVisibility = useMessengerStore(
    (s) => s.toggleAccountBubbleVisibility,
  );

  const setActiveAccountIndex = useMessengerStore(
    (s) => s.setActiveAccountIndex,
  );

  const setOpenChatAccountIndex = useWindowStore(
    (s) => s.setOpenChatAccountIndex,
  );
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);

  const totalConnected = connectedAccounts.length;

  const handleOpenChat = async (account: MessengerAccount) => {
    setActiveAccountIndex(account.index);
    setOpenChatAccountIndex(account.index);
    try {
      await messengerService.openChat(account.index);
    } catch (err) {
      console.warn(
        "[MessengerConnectionCard] Failed to open chat window:",
        err,
      );
    }
  };

  const handleToggleSingleBubble = async (account: MessengerAccount) => {
    const nextVal = !account.isBubbleVisible;
    toggleAccountBubbleVisibility(account.index);
    try {
      await messengerService.toggleBubble(nextVal, account.index);
    } catch (err) {
      console.warn(
        `[MessengerConnectionCard] Failed to toggle bubble for account ${account.index}:`,
        err,
      );
    }
  };

  return (
    <div className="shrink-0 bg-white border border-[#E2E7EF] rounded-[12px] shadow-card overflow-hidden flex flex-col select-none">
      {/* Top Connection Details Area */}
      <div className="p-6 flex items-start justify-between gap-6 flex-wrap lg:flex-nowrap">
        {/* Left: Platform Identity, Master Actions & Account List */}
        <div className="flex flex-col gap-4 flex-1 min-w-0 sm:min-w-[320px]">
          {/* 1. Header with Platform Info and Master Toolbar */}
          <MessengerPlatformHeader
            totalConnected={totalConnected}
            totalAccounts={accounts.length}
            areAnyBubblesVisible={areAnyBubblesVisible}
            onManage={() => setActiveTab("connections")}
          />

          {/* 2. Connected Accounts List (OOP Multi-Account Rows) */}
          <div className="flex flex-col gap-2 pt-1">
            {totalConnected > 0 ? (
              connectedAccounts.map((acc) => (
                <ConnectedAccountRow
                  key={acc.id}
                  account={acc}
                  onOpenChat={handleOpenChat}
                  onToggleBubble={handleToggleSingleBubble}
                />
              ))
            ) : (
              <div className="flex items-center justify-between p-3.5 bg-[#FFF8F2] rounded-[10px] border border-[#FCD34D]/60 gap-3">
                <div className="flex flex-col gap-0.5">
                  <span className="text-[13px] font-semibold text-[#202733]">
                    Chưa có tài khoản Messenger nào được liên kết
                  </span>
                  <span className="text-[11.5px] text-[#D97706]">
                    Đăng nhập tài khoản của bạn để kích hoạt bong bóng chat nổi
                    trên màn hình.
                  </span>
                </div>
                <CommonButton
                  variant="primary"
                  size="sm"
                  onClick={() => setActiveTab("connections")}
                  iconTrailing={<LogIn className="w-3.5 h-3.5" />}
                >
                  Kết nối ngay
                </CommonButton>
              </div>
            )}
          </div>
        </div>

        {/* Right: Metrics Column */}
        <ConnectionMetricsColumn
          visibleBubbleCount={visibleBubbleCount}
          totalConnected={totalConnected}
          unreadCount={totalUnreadCount}
        />
      </div>

      {/* Bottom Reassurance Banner */}
      <div className="bg-[#EAF2FF] px-6 py-2.5 flex items-center gap-2 text-[#0759DB] border-t border-[#E2E7EF]/50">
        <ShieldCheck className="w-4 h-4 shrink-0" />
        <span className="text-[11px] font-normal leading-tight">
          Phiên đăng nhập được lưu trên thiết bị này. Không gửi dữ liệu tài
          khoản ra ngoài.
        </span>
      </div>
    </div>
  );
};
