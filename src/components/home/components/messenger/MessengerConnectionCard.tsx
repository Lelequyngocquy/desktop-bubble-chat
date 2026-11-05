import React, { useMemo } from "react";
import { ShieldCheck, LogIn, FolderOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  messengerService,
  useMessengerStore,
  selectTotalUnreadCount,
  selectVisibleBubbleCount,
  selectAreAnyBubblesVisible,
} from "../../../../platforms/messenger";
import { useWindowStore } from "../../../../stores/windowStore";
import { useNavigationStore } from "../../../../stores/navigationStore";
import { systemService } from "../../../../services/systemService";
import { MessengerAccount } from "../../../../types/account";
import { CommonButton } from "../../../common";
import { MessengerPlatformHeader } from "./MessengerPlatformHeader";
import { ConnectedAccountRow } from "./ConnectedAccountRow";
import { ConnectionMetricsColumn } from "./ConnectionMetricsColumn";

export const MessengerConnectionCard: React.FC = () => {
  const { t } = useTranslation("dashboard");
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
      console.warn("[MessengerConnectionCard] Failed to open chat window:", err);
    }
  };
  const handleToggleSingleBubble = async (account: MessengerAccount) => {
    const nextVal = !account.isBubbleVisible;
    toggleAccountBubbleVisibility(account.index);
    try {
      await messengerService.toggleBubble(nextVal, account.index);
    } catch (err) {
      console.warn(`[MessengerConnectionCard] Failed to toggle bubble for account ${account.index}:`, err);
    }
  };

  return (
    <div className="shrink-0 bg-background-surface border border-border rounded-card shadow-card overflow-hidden flex flex-col select-none">
      <div className="p-6 flex items-stretch justify-between gap-6 flex-wrap lg:flex-nowrap">
        <div className="flex flex-col gap-4 flex-1 min-w-0 sm:min-w-[320px]">
          <MessengerPlatformHeader
            totalConnected={totalConnected}
            totalAccounts={accounts.length}
            areAnyBubblesVisible={areAnyBubblesVisible}
            onManage={() => setActiveTab("connections")}
          />
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
              <div className="flex items-center justify-between p-3.5 bg-status-warningBg rounded-card-sm border border-status-warningBorder gap-3">
                <div className="flex flex-col gap-0.5">
                  <span className="text-body font-semibold text-content-primary">
                    {t("no_accounts_title")}
                  </span>
                  <span className="text-caption text-status-warning">
                    {t("no_accounts_desc")}
                  </span>
                </div>
                <CommonButton
                  variant="primary"
                  size="sm"
                  onClick={() => setActiveTab("connections")}
                  iconTrailing={<LogIn className="w-3.5 h-3.5" />}
                >
                  {t("connect_now")}
                </CommonButton>
              </div>
            )}
          </div>
        </div>
        <ConnectionMetricsColumn
          visibleBubbleCount={visibleBubbleCount}
          totalConnected={totalConnected}
          unreadCount={totalUnreadCount}
        />
      </div>
      <div className="bg-primary-subtle px-6 py-2.5 flex items-center justify-between gap-3 text-primary-foreground border-t border-border flex-wrap sm:flex-nowrap">
        <div className="flex items-center gap-2 min-w-0">
          <ShieldCheck className="w-4 h-4 shrink-0 text-primary" />
          <span className="text-caption font-normal leading-tight">
            {t("reassurance_banner")}
          </span>
        </div>
        <button
          onClick={() => systemService.openProfilesFolder()}
          className="shrink-0 flex items-center gap-1.5 text-caption font-semibold text-primary hover:text-primary-hover hover:underline transition-colors cursor-pointer ml-auto"
          title={t("open_folder_tooltip")}
        >
          <span>{t("open_folder_btn")}</span>
          <FolderOpen className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};

export default MessengerConnectionCard;