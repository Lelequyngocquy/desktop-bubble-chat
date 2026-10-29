import React from "react";
import { PlatformId } from "../../types/platform";
import { MessengerIcon } from "./MessengerIcon";
import { TelegramIcon } from "./TelegramIcon";
import { ZaloIcon } from "./ZaloIcon";
import { DiscordIcon } from "./DiscordIcon";

interface PlatformIconProps {
  platformId: PlatformId;
  className?: string;
  size?: number;
}

export const PlatformIcon: React.FC<PlatformIconProps> = ({ platformId, className, size }) => {
  switch (platformId) {
    case PlatformId.MESSENGER:
      return <MessengerIcon className={className} size={size} />;
    case PlatformId.TELEGRAM:
      return <TelegramIcon className={className} size={size} />;
    case PlatformId.ZALO:
      return <ZaloIcon className={className} size={size} />;
    case PlatformId.DISCORD:
      return <DiscordIcon className={className} size={size} />;
    default:
      return null;
  }
};
