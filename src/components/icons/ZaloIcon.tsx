import React from "react";

interface IconProps {
  className?: string;
  size?: number;
}

export const ZaloIcon: React.FC<IconProps> = ({ className = "text-[13px] font-bold" }) => {
  return <span className={className}>Zalo</span>;
};
