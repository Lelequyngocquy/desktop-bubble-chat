import React from "react";
import { Color, Dimen } from "../../constants/design";

export interface AvatarCircleProps {
  src?: string;
  text?: string;
  icon?: React.ReactNode;
  size?: number; // Size in pixels (default 40px)
  bg?: string; // Background color (hex or class)
  textColor?: string; // Text color (hex or class)
  border?: boolean | string;
  className?: string;
}

export const AvatarCircle: React.FC<AvatarCircleProps> = ({
  src,
  text,
  icon,
  size = Dimen.layout.avatarLg,
  bg = Color.primary.subtle,
  textColor = Color.primary.foreground,
  border = true,
  className = "",
}) => {
  const borderStyle =
    typeof border === "string"
      ? border
      : border
      ? "1px solid rgba(8, 102, 255, 0.12)"
      : "none";
  const isBgHex = bg.startsWith("#") || bg.startsWith("rgb");
  const isTextHex = textColor.startsWith("#") || textColor.startsWith("rgb");
  const fontSize = Math.max(10, Math.round(size * 0.35));
  return (
    <div
      style={{
        width: `${size}px`,
        height: `${size}px`,
        fontSize: `${fontSize}px`,
        backgroundColor: isBgHex ? bg : undefined,
        color: isTextHex ? textColor : undefined,
        border: borderStyle,
      }}
      className={`rounded-pill flex items-center justify-center font-semibold shrink-0 overflow-hidden select-none ${
        !isBgHex ? bg : ""
      } ${!isTextHex ? textColor : ""} ${className}`}
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
        <span>{text}</span>
      )}
    </div>
  );
};