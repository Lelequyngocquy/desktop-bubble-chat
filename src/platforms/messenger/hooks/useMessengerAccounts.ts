import { useState, useCallback } from "react";
import { useMessengerStore, selectIsMessengerConnected } from "../stores/messengerStore";
import { useWindowStore } from "../../../stores/windowStore";
import { messengerService, messengerAuthManager } from "../services";
import { feLog } from "../../../services/loggerService";
import { MessengerAccount } from "../types";

/**
 * Custom hook encapsulating UI actions for Messenger Accounts.
 * Follows Single Responsibility Principle (SRP) by delegating background auth
 * orchestration and timer lifecycles to messengerAuthManager (SSOT).
 */
export function useMessengerAccounts() {
  const {
    accounts,
    activeAccountIndex,
    setActiveAccountIndex,
    addAccount: addAccountToStore,
    removeAccount: removeAccountFromStore,
    updateAccount,
  } = useMessengerStore();

  const isConnected = useMessengerStore(selectIsMessengerConnected);
  const [refreshingId, setRefreshingId] = useState<string | null>(null);

  const handleOpenAccount = useCallback(
    async (account: MessengerAccount) => {
      feLog.info(
        `[UI Click] Người dùng bấm '${account.isConnected ? "Mở chat" : account.isAuthenticating ? "Xem cửa sổ" : "Đăng nhập"}' cho tài khoản: ${account.name} (ID: ${account.id}, Index: ${account.index})`
      );
      setActiveAccountIndex(account.index);

      if (account.isConnected) {
        try {
          feLog.info(`[useMessengerAccounts] Đang mở khung chat cho tài khoản ${account.index}...`);
          useMessengerStore.getState().setAccountBubbleVisibility(account.index, true);
          useWindowStore.getState().setOpenChatAccountIndex(account.index);
          await messengerService.openChat(account.index);
        } catch (err) {
          feLog.error(`[useMessengerAccounts] Failed to open chat window:`, err);
        }
        return;
      }

      // Delegate auth session management to SSOT AuthManager
      await messengerAuthManager.startAuth(account);
    },
    [setActiveAccountIndex]
  );

  const handleCancelAuth = useCallback(
    (account: MessengerAccount) => {
      messengerAuthManager.cancelAuth(account.id);
      updateAccount(account.id, { isAuthenticating: false });
      try {
        messengerService.disconnect(account.index);
      } catch {}
    },
    [updateAccount]
  );

  const handleRefreshAccount = useCallback(
    async (account: MessengerAccount) => {
      setRefreshingId(account.id);
      feLog.info(`[useMessengerAccounts] Người dùng làm mới trạng thái tài khoản ${account.index}...`);
      try {
        await messengerService.refresh(account.index);
        const res = await messengerService.checkAuth(account.index);
        if (res && res.connected) {
          updateAccount(account.id, {
            isConnected: true,
            lastSynced: "Đồng bộ vừa xong",
          });
        }
      } catch (err) {
        feLog.error(`[useMessengerAccounts] Refresh error for account ${account.index}:`, err);
      } finally {
        setTimeout(() => setRefreshingId(null), 800);
      }
    },
    [updateAccount]
  );

  const handleDisconnect = useCallback(
    async (account: MessengerAccount) => {
      feLog.warn(`[useMessengerAccounts] Ngắt kết nối tài khoản: ${account.name} (${account.id})`);
      messengerAuthManager.cancelAuth(account.id);
      try {
        await messengerService.disconnect(account.index);
      } catch (err) {
        feLog.error(`[useMessengerAccounts] Disconnect error:`, err);
      }
      updateAccount(account.id, {
        isConnected: false,
        isAuthenticating: false,
        isBubbleVisible: false,
      });
    },
    [updateAccount]
  );

  const handleAddAccount = useCallback(() => {
    feLog.info("[useMessengerAccounts] Thêm profile Messenger mới...");
    const newAcc = addAccountToStore();
    if (newAcc) {
      feLog.info(`[useMessengerAccounts] Đã tạo tài khoản mới:`, newAcc);
    } else {
      feLog.warn(`[useMessengerAccounts] Không thể thêm tài khoản (đã đạt giới hạn tối đa)`);
    }
    return newAcc;
  }, [addAccountToStore]);

  const handleRemoveAccount = useCallback(
    async (account: MessengerAccount) => {
      feLog.info(`[useMessengerAccounts] Xóa tài khoản: ${account.name} (ID: ${account.id}, Index: ${account.index})`);
      removeAccountFromStore(account.id);
      try {
        await messengerService.toggleBubble(false, account.index);
        await messengerService.clearStorage(account.index);
      } catch (err) {
        feLog.error("[useMessengerAccounts] Failed to clear account storage:", err);
      }
    },
    [removeAccountFromStore]
  );

  return {
    accounts,
    activeAccountIndex,
    isConnected,
    refreshingId,

    // Primary & Alias action handlers for full consumer compatibility
    openAccount: handleOpenAccount,
    handleOpenAccount,

    cancelAuth: handleCancelAuth,
    handleCancelAuth,

    refreshAccount: handleRefreshAccount,
    handleRefresh: handleRefreshAccount,

    removeAccount: handleRemoveAccount,
    handleRemoveAccount,

    addAccount: handleAddAccount,
    handleAddAccount,

    disconnectAccount: handleDisconnect,
    handleDisconnect,

    setActiveAccountIndex,
    updateAccount,
  };
}
