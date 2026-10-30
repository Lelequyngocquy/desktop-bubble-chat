import { create } from "zustand";
import { NavTab } from "../types/navigation";

interface NavigationState {
  activeTab: NavTab;
  setActiveTab: (tab: NavTab) => void;
}

export const useNavigationStore = create<NavigationState>((set) => ({
  activeTab: "home",
  setActiveTab: (tab) => set({ activeTab: tab }),
}));
