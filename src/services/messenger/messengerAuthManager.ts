import { messengerService } from "./messengerService";
import { useMessengerStore } from "../../stores/messengerStore";
import { feLog } from "../loggerService";
import { MessengerAccount } from "../../types/account";

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
      `[AuthManager] Bắt đầu phiên xác thực SSOT cho tài khoản: ${account.name} (ID: ${accountId}, Index: ${account.index})`
    );

    try {
      await messengerService.openAuth(account.index);

      // Polling checkAuth every 1000ms
      const pollInterval = setInterval(async () => {
        // SSOT Invariant: Verify account still exists in store
        const currentAccount = useMessengerStore
          .getState()
          .accounts.find((a) => a.id === accountId);
        if (!currentAccount) {
          // Account was deleted from store! Abort polling immediately
          this.cancelAuth(accountId);
          return;
        }

        try {
          const res = await messengerService.checkAuth(account.index);
          if (res && res.connected) {
            feLog.info(
              `[AuthManager] XÁC THỰC THÀNH CÔNG cho tài khoản ${account.name}! User: ${res.name || ""}`
            );
            this.cancelAuth(accountId);
            useMessengerStore.getState().updateAccount(accountId, {
              isConnected: true,
              isAuthenticating: false,
              isBubbleVisible: true,
              lastSynced: "Đồng bộ vừa xong",
            });
            useMessengerStore.getState().setAccountBubbleVisibility(account.index, true);
            messengerService.toggleBubble(true, account.index).catch(() => {});
          }
        } catch (err) {
          feLog.error(`[AuthManager] Lỗi kiểm tra trạng thái:`, err);
        }
      }, 1000);

      // 120s Timeout
      const timeoutTimer = setTimeout(() => {
        const currentAccount = useMessengerStore
          .getState()
          .accounts.find((a) => a.id === accountId);
        if (currentAccount && currentAccount.isAuthenticating) {
          feLog.warn(`[AuthManager] Quá thời gian chờ đăng nhập (120s) cho ${account.name}`);
          useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
        }
        this.cancelAuth(accountId);
      }, 120000);

      this.activeSessions.set(accountId, { pollInterval, timeoutTimer });
    } catch (err) {
      feLog.error(`[AuthManager] openAuth thất bại:`, err);
      this.cancelAuth(accountId);
      useMessengerStore.getState().updateAccount(accountId, { isAuthenticating: false });
    }
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
