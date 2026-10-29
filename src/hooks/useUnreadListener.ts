import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";

export const useUnreadListener = () => {
  const { setUnreadCount } = useAppStore();

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      try {
        unlisten = await listen<number>("badge-updated", (event) => {
          console.log("[Event] badge-updated received:", event.payload);
          setUnreadCount(event.payload);
        });
      } catch (err) {
        console.log("Event listener not available in browser mode:", err);
      }
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, [setUnreadCount]);

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
