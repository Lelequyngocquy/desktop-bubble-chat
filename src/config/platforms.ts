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
  [PlatformId.TELEGRAM]: {
    id: PlatformId.TELEGRAM,
    name: "Telegram",
    tagline: "More connections coming soon",
    description: "Multi-platform chat heads support.",
    status: PlatformStatus.UPCOMING,
    brandColor: "#229ED9",
    badgeBgColor: "#EEF2F9",
    badgeTextColor: "#6B7483",
    isAvailable: false,
    upcomingBadgeText: "Coming Soon",
  },
  [PlatformId.ZALO]: {
    id: PlatformId.ZALO,
    name: "Zalo",
    tagline: "Stay close to your contacts",
    description: "Direct Zalo messaging wrapper.",
    status: PlatformStatus.UPCOMING,
    brandColor: "#0068FF",
    badgeBgColor: "#EEF2F9",
    badgeTextColor: "#6B7483",
    isAvailable: false,
    upcomingBadgeText: "Coming Soon",
  },
  [PlatformId.DISCORD]: {
    id: PlatformId.DISCORD,
    name: "Discord",
    tagline: "Connect with your communities",
    description: "Discord direct bubble overlay.",
    status: PlatformStatus.UPCOMING,
    brandColor: "#5865F2",
    badgeBgColor: "#EEF2F9",
    badgeTextColor: "#6B7483",
    isAvailable: false,
    upcomingBadgeText: "Coming Soon",
  },
};

export const UPCOMING_PLATFORMS: PlatformConfig[] = [
  PLATFORM_CONFIGS[PlatformId.TELEGRAM],
  PLATFORM_CONFIGS[PlatformId.ZALO],
  PLATFORM_CONFIGS[PlatformId.DISCORD],
];