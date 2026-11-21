import { useState, useCallback, useRef, useEffect } from "react";
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
  const isMountedRef = useRef(true);
  const refreshTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
      clearTimeout(refreshTimerRef.current);
    };
  }, []);

  const handleOpenAccount = useCallback(
    async (account: MessengerAccount) => {
      feLog.info(
        `[UI Click] User clicked '${account.isConnected ? "Open chat" : account.isAuthenticating ? "View window" : "Login"}' for account: ${account.name} (ID: ${account.id}, Index: ${account.index})`
      );
      setActiveAccountIndex(account.index);
      if (account.isConnected) {
        try {
          feLog.info(`[useMessengerAccounts] Opening chat window for account ${account.index}...`);
          useMessengerStore.getState().setAccountBubbleVisibility(account.index, true);
          useWindowStore.getState().setOpenChatAccountIndex(account.index);
          await messengerService.openChat(account.index);
        } catch (err) {
          feLog.error(`[useMessengerAccounts] Failed to open chat window:`, err);
        }
        return;
      }
      await messengerAuthManager.startAuth(account);
    },
    [setActiveAccountIndex]
  );

  const handleCancelAuth = useCallback(
    (account: MessengerAccount) => {
      messengerAuthManager.cancelAuth(account.id);
      updateAccount(account.id, { isAuthenticating: false });
      messengerService.disconnect(account.index).catch((err) => {
        feLog.warn(`[useMessengerAccounts] Cancel auth disconnect error:`, err);
      });
    },
    [updateAccount]
  );

  const handleRefreshAccount = useCallback(
    async (account: MessengerAccount) => {
      setRefreshingId(account.id);
      feLog.info(`[useMessengerAccounts] Refreshing account status ${account.index}...`);
      try {
        await messengerService.refresh(account.index);
        const res = await messengerService.checkAuth(account.index);
        if (isMountedRef.current && res) {
          updateAccount(account.id, {
            isConnected: Boolean(res.connected),
            lastSynced: res.connected ? "Just synced" : "",
          });
        }
      } catch (err) {
        feLog.error(`[useMessengerAccounts] Refresh error for account ${account.index}:`, err);
      } finally {
        clearTimeout(refreshTimerRef.current);
        refreshTimerRef.current = setTimeout(() => {
          if (isMountedRef.current) setRefreshingId(null);
        }, 800);
      }
    },
    [updateAccount]
  );

  const handleDisconnect = useCallback(
    async (account: MessengerAccount) => {
      feLog.warn(`[useMessengerAccounts] Disconnecting account: ${account.name} (${account.id})`);
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
    feLog.info("[useMessengerAccounts] Adding new Messenger profile...");
    const newAcc = addAccountToStore();
    if (newAcc) {
      feLog.info(`[useMessengerAccounts] Created new account:`, newAcc);
    } else {
      feLog.warn(`[useMessengerAccounts] Cannot add account (max limit reached)`);
    }
    return newAcc;
  }, [addAccountToStore]);

  const handleRemoveAccount = useCallback(
    async (account: MessengerAccount) => {
      feLog.info(`[useMessengerAccounts] Removing account: ${account.name} (ID: ${account.id}, Index: ${account.index})`);
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
