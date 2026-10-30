import { PlatformId, PlatformStatus, PlatformConfig } from "../types/platform";

export const PLATFORM_CONFIGS: Record<PlatformId, PlatformConfig> = {
  [PlatformId.MESSENGER]: {
    id: PlatformId.MESSENGER,
    name: "Messenger",
    tagline: "Kết nối trực tiếp qua Smart WebView Wrapper",
    description: "Nhắn tin và gọi video trực tiếp trên nền tảng Messenger của Meta.",
    status: PlatformStatus.DISCONNECTED,
    brandColor: "#0866FF",
    badgeBgColor: "#EAF6EF",
    badgeTextColor: "#25805C",
    isAvailable: true,
  },
  [PlatformId.TELEGRAM]: {
    id: PlatformId.TELEGRAM,
    name: "Telegram",
    tagline: "Sắp có thêm nhiều kết nối",
    description: "Thêm kết nối, thêm tiện lợi.",
    status: PlatformStatus.UPCOMING,
    brandColor: "#229ED9",
    badgeBgColor: "#EEF2F9",
    badgeTextColor: "#6B7483",
    isAvailable: false,
    upcomingBadgeText: "Sắp ra mắt",
  },
  [PlatformId.ZALO]: {
    id: PlatformId.ZALO,
    name: "Zalo",
    tagline: "Gần hơn với mọi liên lạc",
    description: "Gần hơn với mọi liên lạc.",
    status: PlatformStatus.UPCOMING,
    brandColor: "#0068FF",
    badgeBgColor: "#EEF2F9",
    badgeTextColor: "#6B7483",
    isAvailable: false,
    upcomingBadgeText: "Sắp ra mắt",
  },
  [PlatformId.DISCORD]: {
    id: PlatformId.DISCORD,
    name: "Discord",
    tagline: "Luôn bên cạnh cộng đồng",
    description: "Luôn bên cạnh cộng đồng.",
    status: PlatformStatus.UPCOMING,
    brandColor: "#5865F2",
    badgeBgColor: "#EEF2F9",
    badgeTextColor: "#6B7483",
    isAvailable: false,
    upcomingBadgeText: "Sắp ra mắt",
  },
};

export const UPCOMING_PLATFORMS: PlatformConfig[] = [
  PLATFORM_CONFIGS[PlatformId.TELEGRAM],
  PLATFORM_CONFIGS[PlatformId.ZALO],
  PLATFORM_CONFIGS[PlatformId.DISCORD],
];
