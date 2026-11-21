import React from "react";
import { Dimen } from "../../constants/design";

export interface AvatarSquareRoundedProps {
  size?: number;
  radius?: number | string;
  bg?: string;
  textColor?: string;
  icon?: React.ReactNode;
  src?: string;
  text?: string;
  shadow?: boolean | string;
  border?: boolean | string;
  className?: string;
}

export const AvatarSquareRounded: React.FC<AvatarSquareRoundedProps> = ({
  size = 38,
  radius = Dimen.borderRadius.panel,
  bg = "bg-background-app",
  textColor = "text-content-primary",
  icon,
  src,
  text,
  shadow = false,
  border = false,
  className = "",
}) => {
  const borderRadius = typeof radius === "number" ? `${radius}px` : radius;
  const borderClass = border === true ? "border border-border" : "";
  const borderStyle = typeof border === "string" ? border : border === false ? "none" : undefined;
  const isBgHex = bg.startsWith("#") || bg.startsWith("rgb");
  const isTextHex = textColor.startsWith("#") || textColor.startsWith("rgb");
  return (
    <div
      style={{
        width: `${size}px`,
        height: `${size}px`,
        borderRadius,
        backgroundColor: isBgHex ? bg : undefined,
        color: isTextHex ? textColor : undefined,
        border: borderStyle,
      }}
      className={`flex items-center justify-center shrink-0 overflow-hidden select-none ${borderClass} ${
        shadow ? "shadow-sm" : ""
      } ${!isBgHex ? bg : ""} ${!isTextHex ? textColor : ""} ${className}`}
    >
      {src ? (
        <img
          src={src}
          alt={text || "avatar"}
          className="w-full h-full object-cover"
        />
      ) : icon ? (
        icon
      ) : (
        <span className="font-bold text-body">{text}</span>
      )}
    </div>
  );
};
