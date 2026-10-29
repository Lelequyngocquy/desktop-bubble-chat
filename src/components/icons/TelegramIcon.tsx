import React from "react";
import { Send } from "lucide-react";

interface IconProps {
  className?: string;
  size?: number;
}

export const TelegramIcon: React.FC<IconProps> = ({ className = "w-5 h-5", size = 20 }) => {
  return <Send className={className} size={size} />;
};
