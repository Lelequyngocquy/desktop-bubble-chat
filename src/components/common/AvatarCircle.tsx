import React from "react";
import { Dimen } from "../../constants/design";

export interface AvatarCircleProps {
  src?: string;
  text?: string;
  icon?: React.ReactNode;
  size?: number;
  bg?: string;
  textColor?: string;
  border?: boolean | string;
  className?: string;
}

export const AvatarCircle: React.FC<AvatarCircleProps> = ({
  src,
  text,
  icon,
  size = Dimen.layout.avatarLg,
  bg = "bg-primary-subtle",
  textColor = "text-primary-foreground",
  border = true,
  className = "",
}) => {
  const isBgHex = bg.startsWith("#") || bg.startsWith("rgb");
  const isTextHex = textColor.startsWith("#") || textColor.startsWith("rgb");
  const borderClass = border === true ? "border border-primary/15" : "";
  const borderStyle = typeof border === "string" ? border : border === false ? "none" : undefined;
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
      className={`rounded-pill flex items-center justify-center font-semibold shrink-0 overflow-hidden select-none ${borderClass} ${
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
