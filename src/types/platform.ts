export enum PlatformId {
  MESSENGER = "messenger",
}

export enum PlatformStatus {
  CONNECTED = "connected",
  DISCONNECTED = "disconnected",
  CONNECTING = "connecting",
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
}
