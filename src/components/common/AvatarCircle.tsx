import React from "react";

export interface AvatarCircleProps {
  src?: string;
  text?: string;
  icon?: React.ReactNode;
  size?: number; // Kích thước theo pixel (mặc định 40px)
  bg?: string; // Màu nền (hex hoặc class)
  textColor?: string; // Màu chữ (hex hoặc class)
  border?: boolean | string;
  className?: string;
}

export const AvatarCircle: React.FC<AvatarCircleProps> = ({
  src,
  text,
  icon,
  size = 40,
  bg = "#EAF2FF",
  textColor = "#0759DB",
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

  // Tính cỡ chữ tỉ lệ tương ứng với size
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
      className={`rounded-full flex items-center justify-center font-semibold shrink-0 overflow-hidden select-none ${
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
