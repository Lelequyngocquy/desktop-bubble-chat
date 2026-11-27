import { useWindowStore } from "../../stores/windowStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useMessengerStore } from "../../platforms/messenger";
import { windowService } from "../../services/windowService";
import { applyBackendUserSettings } from "../../services/userSettingsSyncService";
import {
  IPC_EVENTS,
  BubbleSpawnedPayload,
  ChatVisibilityPayload,
  ChatWindowResizedPayload,
  UserSettingsSnapshot,
} from "../../types/ipc";
import { UnreadListenerContext } from "./types";

/**
 * Registers desktop window visibility, close confirmation, bubble spawn, and resize listeners.
 */
export async function registerWindowAndLayoutEvents(
  ctx: UnreadListenerContext,
  onRegisterCleanup: (cleanup: () => void) => void,
): Promise<boolean> {
  const { register, isDisposed } = ctx;
  let resizeDebounceTimer: ReturnType<typeof setTimeout> | undefined;
  onRegisterCleanup(() => clearTimeout(resizeDebounceTimer));
  if (!(await register<boolean | ChatVisibilityPayload>(IPC_EVENTS.CHAT_VISIBILITY_CHANGED, (payload) => {
    const windowStore = useWindowStore.getState();
    if (typeof payload === "object" && payload !== null && "visible" in payload) {
      const targetIdx = payload.accountIndex || 1;
      if (payload.visible) {
        windowStore.setOpenChatAccountIndex(targetIdx);
      } else if (windowStore.openChatAccountIndex === targetIdx) {
        windowStore.setOpenChatAccountIndex(null);
      }
    } else if (typeof payload === "boolean") {
      const activeIdx = useMessengerStore.getState().activeAccountIndex;
      if (payload) {
        windowStore.setOpenChatAccountIndex(activeIdx);
      } else if (windowStore.openChatAccountIndex === activeIdx) {
        windowStore.setOpenChatAccountIndex(null);
      }
    }
  }))) return false;
  if (!(await register(IPC_EVENTS.SHOW_CLOSE_DIALOG, () => {
    const action = useSettingsStore.getState().closeWindowAction;
    if (action === "hide_to_tray") {
      windowService.hide();
    } else if (action === "exit_app") {
      windowService.exit();
    } else {
      useWindowStore.getState().setIsCloseDialogOpen(true);
    }
  }))) return false;
  if (!(await register<BubbleSpawnedPayload>(IPC_EVENTS.BUBBLE_SPAWNED, (payload) => {
    if (payload) {
      useWindowStore.getState().setLastBubbleSpawn(payload);
    }
  }))) return false;
  if (!(await register<UserSettingsSnapshot>(IPC_EVENTS.USER_SETTINGS_CHANGED, (settings) => {
    if (settings) {
      applyBackendUserSettings(settings);
    }
  }))) return false;
  return register<ChatWindowResizedPayload>(IPC_EVENTS.CHAT_WINDOW_RESIZED, (data) => {
    if (data && data.accountIndex && data.width && data.height) {
      const currentSetting = useSettingsStore.getState().chatWindowSettings.find(
        (s) => s.accountIndex === data.accountIndex,
      );
      if (currentSetting && currentSetting.mode === "remember_custom") {
        clearTimeout(resizeDebounceTimer);
        resizeDebounceTimer = setTimeout(() => {
          if (!isDisposed()) {
            useSettingsStore.getState().updateChatWindowSize(data.accountIndex, data.width, data.height, true);
          }
        }, 500);
      }
    }
  });
}
