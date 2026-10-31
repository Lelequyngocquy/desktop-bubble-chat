import React, { useState, useEffect } from "react";
import { Minus, Square, Copy, X } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { CloseAppDialog } from "./dialogs/CloseAppDialog";

export const TitleBar: React.FC = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const [isCloseDialogOpen, setIsCloseDialogOpen] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const checkMaximized = async () => {
      try {
        const appWindow = getCurrentWindow();
        setIsMaximized(await appWindow.isMaximized());
        unlisten = await appWindow.onResized(async () => {
          try {
            setIsMaximized(await appWindow.isMaximized());
          } catch {}
        });
      } catch {
        // Fallback in browser preview
      }
    };
    checkMaximized();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleMinimize = async () => {
    try {
      await invoke("minimize_main_window");
    } catch {
      try {
        await getCurrentWindow().minimize();
      } catch {
        console.log("Minimize window fallback");
      }
    }
  };

  const handleToggleMaximize = async () => {
    try {
      const res = await invoke<boolean>("toggle_maximize_main_window");
      setIsMaximized(res);
    } catch {
      try {
        await getCurrentWindow().toggleMaximize();
        setIsMaximized(await getCurrentWindow().isMaximized());
      } catch {
        console.log("Toggle maximize fallback");
      }
    }
  };

  const handleCloseClick = async () => {
    const rememberedAction = localStorage.getItem("dbc_close_window_action");
    if (rememberedAction === "hide_to_tray") {
      try {
        await invoke("hide_main_window");
      } catch {
        try {
          await getCurrentWindow().hide();
        } catch {}
      }
      return;
    }
    if (rememberedAction === "exit_app") {
      try {
        await invoke("exit_app");
      } catch {}
      return;
    }

    setIsCloseDialogOpen(true);
  };

  const handleHideToTray = async () => {
    setIsCloseDialogOpen(false);
    try {
      await invoke("hide_main_window");
    } catch {
      try {
        await getCurrentWindow().hide();
      } catch {}
    }
  };

  const handleExitApp = async () => {
    setIsCloseDialogOpen(false);
    try {
      await invoke("exit_app");
    } catch {
      try {
        await getCurrentWindow().close();
      } catch {}
    }
  };

  return (
    <>
      <header
        data-tauri-drag-region
        onDoubleClick={handleToggleMaximize}
        className="h-[48px] w-full flex items-center justify-between pl-5 pr-0 bg-[#EEF2F9] border-b border-[#E2E7EF] select-none z-50 shrink-0"
      >
        {/* Application Identity (Figma Node #3:1147) */}
        <div
          data-tauri-drag-region
          className="flex items-center gap-[10px] cursor-default"
        >
          <img
            src="/app-icon.png"
            alt="DesktopBubbleChat Logo"
            className="w-[22px] h-[22px] object-contain rounded-[6px] shadow-xs select-none pointer-events-none"
          />
          <span className="text-[12px] font-medium text-[#202733] tracking-tight">
            DesktopBubbleChat - Bong Bóng Trò Chuyện
          </span>
        </div>

        {/* Draggable center area */}
        <div data-tauri-drag-region className="flex-1 h-full" />

        {/* Windows 11 Controls: 3 Buttons (Minimize, Maximize/Restore & Close) */}
        <div className="flex items-stretch h-full">
          {/* Minimize to Taskbar */}
          <button
            onClick={handleMinimize}
            title="Thu nhỏ xuống Taskbar"
            className="w-[46px] h-full flex items-center justify-center text-[#6B7483] hover:text-[#202733] hover:bg-white/60 active:bg-white/90 transition-colors cursor-pointer"
          >
            <Minus className="w-[14px] h-[14px]" />
          </button>

          {/* Fullscreen / Maximize / Restore */}
          <button
            onClick={handleToggleMaximize}
            title={isMaximized ? "Khôi phục kích thước" : "Phóng to toàn màn hình"}
            className="w-[46px] h-full flex items-center justify-center text-[#6B7483] hover:text-[#202733] hover:bg-white/60 active:bg-white/90 transition-colors cursor-pointer"
          >
            {isMaximized ? (
              <Copy className="w-[12px] h-[12px] rotate-180" />
            ) : (
              <Square className="w-[12px] h-[12px]" />
            )}
          </button>

          {/* Close / Hide Options */}
          <button
            onClick={handleCloseClick}
            title="Đóng"
            className="w-[46px] h-full flex items-center justify-center text-[#6B7483] hover:text-white hover:bg-[#E81123] active:bg-[#BF0F1D] transition-colors cursor-pointer"
          >
            <X className="w-[15px] h-[15px]" />
          </button>
        </div>
      </header>

      {/* Confirmation Dialog on Close */}
      <CloseAppDialog
        isOpen={isCloseDialogOpen}
        onClose={() => setIsCloseDialogOpen(false)}
        onHideToTray={handleHideToTray}
        onExitApp={handleExitApp}
      />
    </>
  );
};
