import { PlatformId, PlatformStatus, PlatformConfig } from "../types/platform";

export const PLATFORM_CONFIGS: Record<PlatformId, PlatformConfig> = {
  [PlatformId.MESSENGER]: {
    id: PlatformId.MESSENGER,
    name: "Messenger",
    tagline: "Direct connection via Smart WebView Wrapper",
    description: "Messaging and video calls directly on Meta Messenger platform.",
    status: PlatformStatus.DISCONNECTED,
    brandColor: "#0866FF",
    badgeBgColor: "#EAF6EF",
    badgeTextColor: "#25805C",
    isAvailable: true,
  },
};