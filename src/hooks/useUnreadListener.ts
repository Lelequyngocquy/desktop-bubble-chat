import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";
import { MOCK_DEFAULT_CONNECTED_ACCOUNT } from "../mocks";

interface AuthPayload {
  connected: boolean;
  name?: string | null;
}

export const useUnreadListener = () => {
  const {
    setUnreadCount,
    setIsMessengerConnected,
    setConnectedAccount,
    setBubbleEnabled,
    setIsChatOpen,
  } = useAppStore();

  useEffect(() => {
    let unlistenBadge: (() => void) | undefined;
    let unlistenAuth: (() => void) | undefined;
    let unlistenBubble: (() => void) | undefined;
    let unlistenChat: (() => void) | undefined;

    const setupListeners = async () => {
      try {
        // 1. Lắng nghe cập nhật huy hiệu tin chưa đọc
        unlistenBadge = await listen<number>("badge-updated", (event) => {
          setUnreadCount(event.payload);
        });

        // 2. Lắng nghe trạng thái đăng nhập/xác thực Messenger theo thời gian thực
        unlistenAuth = await listen<AuthPayload>("messenger-auth-status", (event) => {
          const isConnected = event.payload.connected;
          setIsMessengerConnected(isConnected);
          if (isConnected) {
            const name = event.payload.name || MOCK_DEFAULT_CONNECTED_ACCOUNT.name;
            const initials = name
              .split(" ")
              .filter(Boolean)
              .map((n) => n[0])
              .join("")
              .slice(0, 2)
              .toUpperCase() || MOCK_DEFAULT_CONNECTED_ACCOUNT.avatarText;

            setConnectedAccount({
              name,
              avatarText: initials,
              lastSynced: "Đồng bộ vừa xong",
            });
            setBubbleEnabled(true);
          } else {
            setConnectedAccount(null);
            setBubbleEnabled(false);
          }
        });

        // 3. Lắng nghe thay đổi trạng thái hiển thị của bong bóng nổi (Snap dismiss đáy hoặc tray)
        unlistenBubble = await listen<boolean>("bubble-visibility-changed", (event) => {
          setBubbleEnabled(event.payload);
        });

        // 4. Lắng nghe thay đổi trạng thái đóng/mở của khung chat Messenger
        unlistenChat = await listen<boolean>("chat-visibility-changed", (event) => {
          setIsChatOpen(event.payload);
        });
      } catch (err) {
        console.log("Event listeners not available in browser mode:", err);
      }
    };

    setupListeners();

    return () => {
      if (unlistenBadge) unlistenBadge();
      if (unlistenAuth) unlistenAuth();
      if (unlistenBubble) unlistenBubble();
      if (unlistenChat) unlistenChat();
    };
  }, [setUnreadCount, setIsMessengerConnected, setConnectedAccount, setBubbleEnabled, setIsChatOpen]);

  // Helper cho phép giả lập/test số unread
  const simulateUnread = async (count: number) => {
    setUnreadCount(count);
    try {
      await invoke("set_badge_count", { count });
    } catch {
      // Browser preview fallback
    }
  };

  return { simulateUnread };
};
