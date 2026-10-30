import React from "react";
import { useBubbleWindow } from "../../hooks/useBubbleWindow";
import { useBubbleDrag } from "../../hooks/useBubbleDrag";
import { MessengerIcon } from "../icons";

export const BubbleWindow: React.FC = () => {
  const {
    accountIndex,
    unreadCount,
    themeColor,
    isChatOpen,
    handleBubbleClick,
    handleBubbleHoverOpen,
  } = useBubbleWindow();

  const { isDragging, dragOpacity, isNearDismiss, dragProps } = useBubbleDrag({
    onBubbleClick: handleBubbleClick,
    onBubbleHoverOpen: handleBubbleHoverOpen,
    isChatOpen,
    accountIndex,
  });

  return (
    <div
      {...dragProps}
      style={{ opacity: dragOpacity }}
      className={`w-16 h-16 relative flex items-center justify-center bg-transparent select-none cursor-grab active:cursor-grabbing transition-all duration-75 ${
        isNearDismiss ? "scale-90" : isDragging ? "scale-95" : "scale-100"
      }`}
    >
      {/* 1. Main Round Bubble Body (56x56) */}
      <div
        style={{
          background: `linear-gradient(180deg, ${themeColor.hex}, ${themeColor.borderHex})`,
        }}
        className={`w-[56px] h-[56px] rounded-full flex items-center justify-center text-white border-2 border-white relative shrink-0 transition-all duration-150 hover:brightness-105 active:scale-95 overflow-hidden shadow-bubble ${
          isNearDismiss ? "brightness-90 ring-2 ring-red-400/50" : ""
        }`}
      >
        {/* Messenger Logo SVG */}
        <MessengerIcon className="w-[30px] h-[30px] fill-white pointer-events-none drop-shadow-xs" />
      </div>

      {/* 2. Unread Badge Counter (Top Right) */}
      {unreadCount > 0 && (
        <div className="absolute top-0 right-0 h-[20px] min-w-[20px] px-1 bg-[#E81123] text-white text-[10px] font-bold rounded-full flex items-center justify-center border-2 border-white pointer-events-none animate-in fade-in zoom-in duration-200">
          <span>{unreadCount > 99 ? "99+" : unreadCount}</span>
        </div>
      )}
    </div>
  );
};

export default BubbleWindow;
