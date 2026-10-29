import React from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/appStore";
import { useBubbleDrag } from "../../hooks/useBubbleDrag";
import { MessengerIcon } from "../icons";

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

  const { isDragging, dragOpacity, isNearDismiss, dragProps } = useBubbleDrag({
    onBubbleClick: handleBubbleClick,
    isChatOpen,
  });

  return (
    <div
      {...dragProps}
      style={{ opacity: dragOpacity }}
      className={`w-16 h-16 relative flex items-center justify-center bg-transparent select-none cursor-grab active:cursor-grabbing transition-all duration-75 ${
        isNearDismiss ? "scale-90" : isDragging ? "scale-95" : "scale-100"
      }`}
    >
      {/* 1. Main Round Bubble Body (56x56) - Làm mờ và thu nhỏ khi kéo xuống đáy để báo hiệu ẩn */}
      <div
        className={`w-[56px] h-[56px] rounded-full bg-gradient-to-b from-[#0084FF] to-[#0066FF] flex items-center justify-center text-white border-2 border-white relative shrink-0 transition-all duration-150 hover:brightness-105 active:scale-95 overflow-hidden ${
          isNearDismiss ? "brightness-90 ring-2 ring-red-400/50" : ""
        }`}
      >
        {/* Messenger Logo SVG - Không drop-shadow */}
        <MessengerIcon className="w-[30px] h-[30px] fill-white pointer-events-none" />
      </div>

      {/* 2. Unread Badge Counter (Top Right) - Viền trắng stroke, không shadow */}
      {unreadCount > 0 && (
        <div className="absolute top-0 right-0 h-[20px] min-w-[20px] px-1 bg-[#E81123] text-white text-[10px] font-bold rounded-full flex items-center justify-center border-2 border-white pointer-events-none animate-in fade-in zoom-in duration-200">
          <span>{unreadCount > 99 ? "99+" : unreadCount}</span>
        </div>
      )}
    </div>
  );
};

export default BubbleWindow;
