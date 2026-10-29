import React from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/appStore";
import { useBubbleDrag } from "../../hooks/useBubbleDrag";

export const BubbleWindow: React.FC = () => {
  const { unreadCount, isChatOpen, setIsChatOpen } = useAppStore();

  const handleBubbleClick = async () => {
    try {
      const isOpen = await invoke<boolean>("toggle_chat");
      setIsChatOpen(isOpen);
    } catch {
      // In standard browser dev preview
      setIsChatOpen(!isChatOpen);
    }
  };

  const { isDragging, dragProps } = useBubbleDrag({
    onBubbleClick: handleBubbleClick,
  });

  return (
    <div
      {...dragProps}
      className={`w-16 h-16 relative flex items-center justify-center bg-transparent select-none overflow-visible cursor-grab active:cursor-grabbing transition-transform ${
        isDragging ? "scale-95" : "scale-100"
      }`}
    >
      {/* 1. Main Round Bubble Avatar Body (58x58) */}
      <div
        className={`w-[58px] h-[58px] rounded-full bg-gradient-to-b from-[#0084FF] to-[#0066FF] flex items-center justify-center text-white border-2 border-white/30 relative shrink-0 transition-all duration-150 ${
          isDragging
            ? "shadow-[0_8px_24px_rgba(0,106,255,0.6)]"
            : "shadow-[0_4px_18px_rgba(0,106,255,0.45)] hover:scale-105 active:scale-95"
        }`}
      >
        {/* Messenger Logo SVG */}
        <svg
          viewBox="0 0 36 36"
          className="w-[32px] h-[32px] fill-white drop-shadow-xs pointer-events-none"
        >
          <path d="M18 1C8.61 1 1 8.16 1 17c0 5.04 2.5 9.54 6.43 12.45V35l5.29-2.91c1.68.47 3.46.72 5.28.72 9.39 0 17-7.16 17-16S27.39 1 18 1zm1.75 21.62l-4.34-4.63-8.48 4.63 9.33-9.91 4.45 4.63 8.37-4.63-9.33 9.91z" />
        </svg>
      </div>

      {/* 2. Unread Badge Counter (Top Right) */}
      {unreadCount > 0 && (
        <div className="absolute -top-0.5 -right-0.5 h-[20px] min-w-[20px] px-1 bg-[#E81123] text-white text-[10px] font-bold rounded-full flex items-center justify-center border-2 border-white shadow-sm pointer-events-none animate-in fade-in zoom-in duration-200">
          <span>{unreadCount > 99 ? "99+" : unreadCount}</span>
        </div>
      )}
    </div>
  );
};

export default BubbleWindow;
