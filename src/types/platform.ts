export enum PlatformId {
  MESSENGER = "messenger",
  TELEGRAM = "telegram",
  ZALO = "zalo",
  DISCORD = "discord",
}

export enum PlatformStatus {
  CONNECTED = "connected",
  DISCONNECTED = "disconnected",
  CONNECTING = "connecting",
  UPCOMING = "upcoming",
}

export interface PlatformConfig {
  id: PlatformId;
  name: string;
  tagline: string;
  description: string;
  status: PlatformStatus;
  brandColor: string;
  badgeBgColor: string;
  badgeTextColor: string;
  isAvailable: boolean;
  upcomingBadgeText?: string;
}
