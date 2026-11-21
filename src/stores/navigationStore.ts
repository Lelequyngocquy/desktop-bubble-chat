import { create } from "zustand";
import { NavTab } from "../types/navigation";

interface NavigationState {
  activeTab: NavTab;
  pendingScrollTarget: string | null;
  setActiveTab: (tab: NavTab) => void;
  navigateToSection: (tab: NavTab, elementId: string) => void;
  clearPendingScrollTarget: () => void;
}

export const useNavigationStore = create<NavigationState>((set) => ({
  activeTab: "home",
  pendingScrollTarget: null,
  setActiveTab: (tab) => set({ activeTab: tab }),
  navigateToSection: (tab, elementId) => set({ activeTab: tab, pendingScrollTarget: elementId }),
  clearPendingScrollTarget: () => set({ pendingScrollTarget: null }),
}));
