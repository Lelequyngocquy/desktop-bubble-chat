import { messengerService } from "./messengerService";
import { useMessengerStore } from "../stores/messengerStore";
import { feLog } from "../../../services/loggerService";
import { MessengerAccount } from "../types";

interface ActiveSession {
  token: number;
  cancelled: boolean;
  polling: boolean;
  pollInterval?: ReturnType<typeof setInterval>;
  timeoutTimer?: ReturnType<typeof setTimeout>;
}

/**
 * Single Source of Truth (SSOT) manager for Messenger Authentication lifecycles.
 */
class MessengerAuthManager {
  private activeSessions = new Map<string, ActiveSession>();
  private nextToken = 0;

  async startAuth(account: MessengerAccount): Promise<void> {
    const accountId = account.id;
    this.cancelAuth(accountId);
    const session: ActiveSession = {
      token: ++this.nextToken,
      cancelled: false,
      polling: false,
    };
    this.activeSessions.set(accountId, session);
    useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: true });
    feLog.info(
      `[AuthManager] Starting auth session for account: ${account.name} (ID: ${accountId}, Index: ${account.index})`,
    );
    try {
      await messengerService.openAuth(account.index);
      if (session.cancelled || this.activeSessions.get(accountId) !== session) {
        return;
      }
      session.pollInterval = setInterval(async () => {
        if (session.cancelled || session.polling || this.activeSessions.get(accountId) !== session) {
          return;
        }
        const currentAccount = useMessengerStore.getState().accounts.find((a) => a.id === accountId);
        if (!currentAccount) {
          this.cancelAuth(accountId);
          return;
        }
        session.polling = true;
        try {
          const res = await messengerService.checkAuth(account.index);
          if (session.cancelled || this.activeSessions.get(accountId) !== session) {
            return;
          }
          if (res && res.connected) {
            feLog.info(
              `[AuthManager] Authentication successful for account ${account.name}! User: ${res.name || ""}`,
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
        } finally {
          session.polling = false;
        }
      }, 1000);
      session.timeoutTimer = setTimeout(() => {
        if (session.cancelled || this.activeSessions.get(accountId) !== session) {
          return;
        }
        const currentAccount = useMessengerStore.getState().accounts.find((a) => a.id === accountId);
        if (currentAccount && currentAccount.isAuthenticating) {
          feLog.warn(`[AuthManager] Login timeout (120s) for ${account.name}`);
          useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
        }
        this.cancelAuth(accountId);
      }, 120000);
    } catch (err) {
      feLog.error(`[AuthManager] openAuth failed:`, err);
      this.cancelAuth(accountId);
      useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
    }
  }

  hasSession(accountId: string): boolean {
    const session = this.activeSessions.get(accountId);
    return Boolean(session && !session.cancelled);
  }

  cancelAuth(accountId: string): void {
    const session = this.activeSessions.get(accountId);
    if (session) {
      session.cancelled = true;
      if (session.pollInterval) clearInterval(session.pollInterval);
      if (session.timeoutTimer) clearTimeout(session.timeoutTimer);
      this.activeSessions.delete(accountId);
    }
  }

  cancelAll(): void {
    for (const session of this.activeSessions.values()) {
      session.cancelled = true;
      if (session.pollInterval) clearInterval(session.pollInterval);
      if (session.timeoutTimer) clearTimeout(session.timeoutTimer);
    }
    this.activeSessions.clear();
  }
}

export const messengerAuthManager = new MessengerAuthManager();
