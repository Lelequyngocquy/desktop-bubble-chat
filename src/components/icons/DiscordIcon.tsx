import React from "react";
import { Gamepad2 } from "lucide-react";

interface IconProps {
  className?: string;
  size?: number;
}

export const DiscordIcon: React.FC<IconProps> = ({ className = "w-5 h-5", size = 20 }) => {
  return <Gamepad2 className={className} size={size} />;
};
