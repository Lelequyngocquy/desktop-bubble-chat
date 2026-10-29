import React, { useState, useEffect } from "react";
import { MessageSquare, Minus, Square, Copy, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AvatarSquareRounded } from "../common";

export const TitleBar: React.FC = () => {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const checkMaximized = async () => {
      try {
        const appWindow = getCurrentWindow();
        setIsMaximized(await appWindow.isMaximized());
        unlisten = await appWindow.onResized(async () => {
          setIsMaximized(await appWindow.isMaximized());
        });
      } catch {
        // Ignored in standard browser preview
      }
    };
    checkMaximized();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleMinimize = async () => {
    try {
      await getCurrentWindow().minimize();
    } catch {
      console.log("Minimize window");
    }
  };

  const handleToggleMaximize = async () => {
    try {
      await getCurrentWindow().toggleMaximize();
    } catch {
      console.log("Toggle maximize window");
    }
  };

  const handleClose = async () => {
    try {
      await getCurrentWindow().close();
    } catch {
      console.log("Close window");
    }
  };

  return (
    <header
      data-tauri-drag-region
      className="h-[48px] w-full flex items-center justify-between pl-5 pr-0 bg-[#EEF2F9] border-b border-[#E2E7EF] select-none z-50 shrink-0"
    >
      {/* Application Identity (Figma Node #3:1147) */}
      <div
        data-tauri-drag-region
        className="flex items-center gap-[10px] cursor-default"
      >
        <AvatarSquareRounded
          size={24}
          radius={7}
          bg="#0866FF"
          shadow
          icon={<MessageSquare className="w-[14px] h-[14px] text-white stroke-[2.2]" />}
        />
        <span className="text-[12px] font-medium text-[#202733] tracking-tight">
          DesktopBubbleChat
        </span>
      </div>

      {/* Draggable center area */}
      <div data-tauri-drag-region className="flex-1 h-full" />

      {/* Windows 11 Controls (Figma Node #3:1151) */}
      <div className="flex items-stretch h-full">
        {/* Minimize */}
        <button
          onClick={handleMinimize}
          title="Thu nhỏ"
          className="w-[46px] h-full flex items-center justify-center text-[#6B7483] hover:text-[#202733] hover:bg-white/60 active:bg-white/90 transition-colors"
        >
          <Minus className="w-[14px] h-[14px]" />
        </button>

        {/* Maximize / Restore */}
        <button
          onClick={handleToggleMaximize}
          title={isMaximized ? "Khôi phục" : "Phóng to"}
          className="w-[46px] h-full flex items-center justify-center text-[#6B7483] hover:text-[#202733] hover:bg-white/60 active:bg-white/90 transition-colors"
        >
          {isMaximized ? (
            <Copy className="w-[12px] h-[12px] rotate-180" />
          ) : (
            <Square className="w-[12px] h-[12px]" />
          )}
        </button>

        {/* Close */}
        <button
          onClick={handleClose}
          title="Đóng"
          className="w-[46px] h-full flex items-center justify-center text-[#6B7483] hover:text-white hover:bg-[#E81123] active:bg-[#BF0F1D] transition-colors"
        >
          <X className="w-[15px] h-[15px]" />
        </button>
      </div>
    </header>
  );
};
