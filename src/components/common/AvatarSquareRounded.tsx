import React from "react";

export interface AvatarSquareRoundedProps {
  size?: number; // Kích thước pixel cạnh vuông (mặc định 38px)
  radius?: number | string; // Bán kính bo góc (mặc định 8px)
  bg?: string; // Màu nền (hex hoặc class)
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
  radius = 8,
  bg = "#EEF2F9",
  textColor = "#202733",
  icon,
  src,
  text,
  shadow = false,
  border = false,
  className = "",
}) => {
  const borderRadius = typeof radius === "number" ? `${radius}px` : radius;
  const borderStyle =
    typeof border === "string"
      ? border
      : border
      ? "1px solid #E2E7EF"
      : "none";

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
      className={`flex items-center justify-center shrink-0 overflow-hidden select-none ${
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
        <span className="font-bold text-[13px]">{text}</span>
      )}
    </div>
  );
};
