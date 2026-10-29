import { create } from "zustand";

interface WindowState {
  isChatOpen: boolean;
  windowLabel: string;
  setIsChatOpen: (isOpen: boolean) => void;
  setWindowLabel: (label: string) => void;
}

export const useWindowStore = create<WindowState>((set) => ({
  isChatOpen: false,
  windowLabel: "main",
  setIsChatOpen: (isOpen) => set({ isChatOpen: isOpen }),
  setWindowLabel: (label) => set({ windowLabel: label }),
}));
