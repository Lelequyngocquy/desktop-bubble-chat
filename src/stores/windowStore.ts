import { create } from "zustand";
import { BubbleSpawnedPayload } from "../types/ipc";

interface WindowState {
  // SSOT: Account index of the currently open chat window (1..5), or null if closed
  openChatAccountIndex: number | null;
  windowLabel: string;

  // Compatibility getter
  isChatOpen: boolean;

  // Dialog & Spawn event state
  isCloseDialogOpen: boolean;
  lastBubbleSpawn: BubbleSpawnedPayload | null;

  setOpenChatAccountIndex: (index: number | null) => void;
  setIsChatOpen: (isOpen: boolean, accountIndex?: number) => void;
  setWindowLabel: (label: string) => void;
  setIsCloseDialogOpen: (open: boolean) => void;
  setLastBubbleSpawn: (payload: BubbleSpawnedPayload) => void;
}

export const useWindowStore = create<WindowState>((set, get) => ({
  openChatAccountIndex: null,
  windowLabel: "main",
  isChatOpen: false,
  isCloseDialogOpen: false,
  lastBubbleSpawn: null,

  setOpenChatAccountIndex: (index) =>
    set({
      openChatAccountIndex: index,
      isChatOpen: index !== null,
    }),

  setIsChatOpen: (isOpen, accountIndex) => {
    if (isOpen) {
      const idx = accountIndex !== undefined ? accountIndex : (get().openChatAccountIndex || 1);
      set({ openChatAccountIndex: idx, isChatOpen: true });
    } else {
      set({ openChatAccountIndex: null, isChatOpen: false });
    }
  },

  setWindowLabel: (label) => set({ windowLabel: label }),
  setIsCloseDialogOpen: (open) => set({ isCloseDialogOpen: open }),
  setLastBubbleSpawn: (payload) => set({ lastBubbleSpawn: payload }),
}));

// --- Pure Selectors ---
export const selectOpenChatAccountIndex = (state: WindowState): number | null =>
  state.openChatAccountIndex;

export const selectIsAnyChatOpen = (state: WindowState): boolean =>
  state.openChatAccountIndex !== null;

export const selectIsAccountChatOpen = (state: WindowState, accountIndex: number): boolean =>
  state.openChatAccountIndex === accountIndex;
