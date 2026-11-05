import { messengerService } from "./messengerService";
import { useMessengerStore } from "../stores/messengerStore";
import { feLog } from "../../../services/loggerService";
import { MessengerAccount } from "../types";

interface ActiveSession {
  pollInterval: ReturnType<typeof setInterval>;
  timeoutTimer: ReturnType<typeof setTimeout>;
}

/**
 * Single Source of Truth (SSOT) manager for Messenger Authentication lifecycles.
 * Decouples async timers, polling, and timeouts completely from React component lifecycles.
 * Guarantees zero orphaned timers when components unmount or accounts are deleted.
 */
class MessengerAuthManager {
  private activeSessions = new Map<string, ActiveSession>();

  /**
   * Start an authentication session for an account.
   * Cancels any prior timers for this account to guarantee SSOT.
   */
  async startAuth(account: MessengerAccount): Promise<void> {
    const accountId = account.id;
    this.cancelAuth(accountId);
    const store = useMessengerStore.getState();
    store.updateAccount(accountId, { isAuthenticating: true });
    feLog.info(
      `[AuthManager] Starting auth session for account: ${account.name} (ID: ${accountId}, Index: ${account.index})`
    );
    try {
      await messengerService.openAuth(account.index);
      const pollInterval = setInterval(async () => {
        const currentAccount = useMessengerStore
          .getState()
          .accounts.find((a) => a.id === accountId);
        if (!currentAccount) {
          this.cancelAuth(accountId);
          return;
        }
        try {
          const res = await messengerService.checkAuth(account.index);
          if (res && res.connected) {
            feLog.info(
              `[AuthManager] Authentication successful for account ${account.name}! User: ${res.name || ""}`
            );
            this.cancelAuth(accountId);
            useMessengerStore.getState().updateAccount(accountId, {
              isConnected: true,
              isAuthenticating: false,
              isBubbleVisible: true,
              lastSynced: "Just synced",
            });
            useMessengerStore.getState().setAccountBubbleVisibility(account.index, true);
            messengerService.toggleBubble(true, account.index).catch(() => {});
          }
        } catch (err) {
          feLog.error(`[AuthManager] Status check error:`, err);
        }
      }, 1000);

      const timeoutTimer = setTimeout(() => {
        const currentAccount = useMessengerStore
          .getState()
          .accounts.find((a) => a.id === accountId);
        if (currentAccount && currentAccount.isAuthenticating) {
          feLog.warn(`[AuthManager] Login timeout (120s) for ${account.name}`);
          useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
        }
        this.cancelAuth(accountId);
      }, 120000);

      this.activeSessions.set(accountId, { pollInterval, timeoutTimer });
    } catch (err) {
      feLog.error(`[AuthManager] openAuth failed:`, err);
      this.cancelAuth(accountId);
      useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
    }
  }

  /**
   * Check if an active authentication session exists for an account.
   */
  hasSession(accountId: string): boolean {
    return this.activeSessions.has(accountId);
  }

  /**
   * Cancel and cleanup any running polling or timeout for an account.
   */
  cancelAuth(accountId: string): void {
    const session = this.activeSessions.get(accountId);
    if (session) {
      clearInterval(session.pollInterval);
      clearTimeout(session.timeoutTimer);
      this.activeSessions.delete(accountId);
    }
  }

  /**
   * Cancel all running sessions across all accounts.
   */
  cancelAll(): void {
    for (const session of this.activeSessions.values()) {
      clearInterval(session.pollInterval);
      clearTimeout(session.timeoutTimer);
    }
    this.activeSessions.clear();
  }
}

export const messengerAuthManager = new MessengerAuthManager();
