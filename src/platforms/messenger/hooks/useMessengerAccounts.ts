import { useCallback } from "react";
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
  const accounts = useMessengerStore((s) => s.accounts);
  const activeAccountIndex = useMessengerStore((s) => s.activeAccountIndex);
  const isConnected = useMessengerStore(selectIsMessengerConnected);
  const { setActiveAccountIndex, addAccount: addAccountToStore, removeAccount: removeAccountFromStore, updateAccount } = useMessengerStore.getState();

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
    openAccount: handleOpenAccount,
    cancelAuth: handleCancelAuth,
    removeAccount: handleRemoveAccount,
    addAccount: handleAddAccount,
    disconnectAccount: handleDisconnect,
    setActiveAccountIndex,
    updateAccount,
  };
}
