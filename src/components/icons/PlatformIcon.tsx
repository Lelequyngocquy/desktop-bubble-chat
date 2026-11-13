import React from "react";
import { PlatformId } from "../../types/platform";
import { MessengerIcon } from "./MessengerIcon";

interface PlatformIconProps {
  platformId: PlatformId;
  className?: string;
  size?: number;
}

export const PlatformIcon: React.FC<PlatformIconProps> = ({ platformId, className, size }) => {
  switch (platformId) {
    case PlatformId.MESSENGER:
      return <MessengerIcon className={className} size={size} />;
    default:
      return null;
  }
};
