/**
 * Service managing Desktop Notification dispatch (SRP).
 */
export const notificationService = {
  /**
   * Checks or requests notification permissions from the OS / Webview.
   */
  async requestPermission(): Promise<boolean> {
    if (typeof window === "undefined" || !("Notification" in window)) {
      return false;
    }
    try {
      if (Notification.permission === "granted") {
        return true;
      }
      if (Notification.permission !== "denied") {
        const res = await Notification.requestPermission();
        return res === "granted";
      }
    } catch (e) {
      console.warn("[NotificationService] Permission request failed:", e);
    }
    return false;
  },

  /**
   * Sends a native desktop toast notification.
   */
  sendNotification(title: string, options?: NotificationOptions): void {
    if (typeof window === "undefined" || !("Notification" in window)) {
      return;
    }
    try {
      if (Notification.permission === "granted") {
        new Notification(title, options);
      }
    } catch (e) {
      console.warn("[NotificationService] Failed to send notification:", e);
    }
  },
};
