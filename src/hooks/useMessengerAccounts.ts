import { useState, useCallback } from "react";
import { useMessengerStore, selectIsMessengerConnected } from "../stores/messengerStore";
import { messengerService, messengerAuthManager } from "../services/messenger";
import { feLog } from "../services/loggerService";
import { MessengerAccount } from "../types/account";

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
    addAccount,
    removeAccount,
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
        console.warn("[useMessengerAccounts] Failed to refresh account:", err);
      } finally {
        setTimeout(() => setRefreshingId(null), 800);
      }
    },
    [updateAccount]
  );

  const handleRemoveAccount = useCallback(
    async (account: MessengerAccount) => {
      feLog.info(`[UI Click] Xóa tài khoản: ${account.name} (ID: ${account.id}, Index: ${account.index})`);
      removeAccount(account.id);
      try {
        await messengerService.clearStorage(account.index);
      } catch (err) {
        console.error("[useMessengerAccounts] Failed to clear account storage:", err);
      }
    },
    [removeAccount]
  );

  const handleAddAccount = useCallback(() => {
    feLog.info(`[UI Click] Người dùng bấm 'Thêm tài khoản Messenger'`);
    const newAcc = addAccount();
    if (newAcc) {
      feLog.info(`[useMessengerAccounts] Đã tạo tài khoản mới:`, newAcc);
    } else {
      feLog.warn(`[useMessengerAccounts] Không thể thêm tài khoản (đã đạt giới hạn tối đa)`);
    }
    return newAcc;
  }, [addAccount]);

  return {
    accounts,
    activeAccountIndex,
    isConnected,
    refreshingId,
    openAccount: handleOpenAccount,
    cancelAuth: handleCancelAuth,
    refreshAccount: handleRefreshAccount,
    removeAccount: handleRemoveAccount,
    addAccount: handleAddAccount,
  };
}
