import { create } from "zustand";
import { MOCK_FALLBACK_USER } from "../mocks";

interface SystemUserState {
  systemUser: string;
  systemUserInitials: string;
  setSystemUser: (name: string) => void;
}

function calculateInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return MOCK_FALLBACK_USER.avatarText;
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  const first = parts[0][0];
  const last = parts[parts.length - 1][0];
  return (first + last).toUpperCase();
}

export const useSystemUserStore = create<SystemUserState>((set) => ({
  systemUser: MOCK_FALLBACK_USER.name,
  systemUserInitials: MOCK_FALLBACK_USER.avatarText,

  setSystemUser: (name: string) => {
    const cleanName = name.trim();
    if (!cleanName) {
      set({
        systemUser: MOCK_FALLBACK_USER.name,
        systemUserInitials: MOCK_FALLBACK_USER.avatarText,
      });
      return;
    }
    set({
      systemUser: cleanName,
      systemUserInitials: calculateInitials(cleanName),
    });
  },
}));
