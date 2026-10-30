import React, { useState } from "react";
import { useBubbleWindow, useBubbleDrag } from "../../hooks/bubble";
import { useSettingsStore, BubbleSize } from "../../stores/settingsStore";
import { useMessengerStore } from "../../stores/messengerStore";
import { MessengerIcon } from "../icons";

const BUBBLE_STYLE_MAP: Record<
  BubbleSize,
  { circle: string; icon: string; badge: string }
> = {
  sm: {
    circle: "w-[40px] h-[40px]",
    icon: "w-[20px] h-[20px]",
    badge: "h-[16px] min-w-[16px] px-0.5 text-[9px] top-0.5 right-0.5",
  },
  md: {
    circle: "w-[48px] h-[48px]",
    icon: "w-[24px] h-[24px]",
    badge: "h-[18px] min-w-[18px] px-1 text-[10px] top-0.5 right-0.5",
  },
  lg: {
    circle: "w-[56px] h-[56px]",
    icon: "w-[28px] h-[28px]",
    badge: "h-[20px] min-w-[20px] px-1 text-[10px] top-0.5 right-0.5",
  },
};

export const BubbleWindow: React.FC = () => {
  const {
    accountIndex,
    unreadCount,
    themeColor,
    isChatOpen,
  } = useBubbleWindow();

  const account = useMessengerStore((s) => s.accounts.find((a) => a.index === accountIndex));
  const hasPulse = account?.hasNewMessagePulse ?? false;
  const avatarUrl = account?.latestSenderAvatar;
  const [imageError, setImageError] = useState(false);

  const bubbleSize = useSettingsStore((s) => s.bubbleSize);
  const sizeStyle = BUBBLE_STYLE_MAP[bubbleSize] ?? BUBBLE_STYLE_MAP.md;

  const { isDragging, dragOpacity, isNearDismiss, dragProps } = useBubbleDrag({
    isChatOpen,
    accountIndex,
  });

  return (
    <div
      {...dragProps}
      style={{ opacity: dragOpacity }}
      className="w-full h-full relative flex items-center justify-center bg-transparent select-none cursor-grab active:cursor-grabbing"
    >
      {/* 0. New Message Pulse / Ripple Effect (Bước 3.1) */}
      {hasPulse && (
        <span
          className="absolute inset-0 rounded-full animate-ping opacity-60 pointer-events-none"
          style={{ backgroundColor: themeColor.hex }}
        />
      )}

      {/* 1. Main Round Bubble Body */}
      <div
        style={{
          background: `linear-gradient(180deg, ${themeColor.hex}, ${themeColor.borderHex})`,
          boxShadow: hasPulse ? `0 0 16px ${themeColor.hex}` : undefined,
        }}
        className={`${sizeStyle.circle} rounded-full flex items-center justify-center text-white border-2 border-white relative shrink-0 transition-transform duration-150 hover:brightness-105 overflow-hidden shadow-bubble ${
          isNearDismiss ? "scale-90 brightness-90 ring-2 ring-red-400/50" : isDragging ? "scale-95" : "scale-100"
        } ${hasPulse ? "ring-2 ring-white" : ""}`}
      >
        {/* Avatar Image (Bước 3.2) or Messenger Logo SVG */}
        {avatarUrl && !imageError ? (
          <img
            src={avatarUrl}
            alt={account?.name || "Avatar"}
            className="w-full h-full object-cover pointer-events-none"
            onError={() => setImageError(true)}
          />
        ) : (
          <MessengerIcon className={`${sizeStyle.icon} fill-white pointer-events-none drop-shadow-xs`} />
        )}
      </div>

      {/* 2. Unread Badge Counter (Top Right) */}
      {unreadCount > 0 && (
        <div className={`absolute ${sizeStyle.badge} bg-[#E81123] text-white font-bold rounded-full flex items-center justify-center border-2 border-white pointer-events-none animate-in fade-in zoom-in duration-200`}>
          <span>{unreadCount > 99 ? "99+" : unreadCount}</span>
        </div>
      )}
    </div>
  );
};

export default BubbleWindow;
