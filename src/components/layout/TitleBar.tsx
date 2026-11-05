import React, { useState, useEffect } from "react";
import { Minus, Square, Copy, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { windowService } from "../../services/windowService";
import { useSettingsStore } from "../../stores/settingsStore";
import { useWindowStore } from "../../stores/windowStore";
import { CloseAppDialog } from "./dialogs/CloseAppDialog";

export const TitleBar: React.FC = () => {
  const { t } = useTranslation("common");
  const [isMaximized, setIsMaximized] = useState(false);
  const isCloseDialogOpen = useWindowStore((s) => s.isCloseDialogOpen);
  const setIsCloseDialogOpen = useWindowStore((s) => s.setIsCloseDialogOpen);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const checkMaximized = async () => {
      try {
        setIsMaximized(await windowService.isMaximized());
        unlisten = await windowService.onResized(async () => {
          try {
            setIsMaximized(await windowService.isMaximized());
          } catch {}
        });
      } catch {}
    };
    checkMaximized();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleMinimize = async () => {
    await windowService.minimize();
  };

  const handleToggleMaximize = async () => {
    const res = await windowService.toggleMaximize();
    setIsMaximized(res);
  };

  const handleCloseClick = async () => {
    const rememberedAction = useSettingsStore.getState().closeWindowAction;
    if (rememberedAction === "hide_to_tray") {
      await windowService.hide();
      return;
    }
    if (rememberedAction === "exit_app") {
      await windowService.exit();
      return;
    }
    setIsCloseDialogOpen(true);
  };

  const handleHideToTray = async () => {
    setIsCloseDialogOpen(false);
    await windowService.hide();
  };

  const handleExitApp = async () => {
    setIsCloseDialogOpen(false);
    await windowService.exit();
  };

  return (
    <>
      <header
        data-tauri-drag-region
        onDoubleClick={handleToggleMaximize}
        className="h-titlebar w-full flex items-center justify-between pl-5 pr-0 bg-background-app border-b border-border select-none z-50 shrink-0"
      >
        <div
          data-tauri-drag-region
          className="flex items-center gap-[10px] cursor-default"
        >
          <img
            src="/app-icon.png"
            alt="DesktopBubbleChat Logo"
            className="w-[22px] h-[22px] object-contain rounded-control shadow-xs select-none pointer-events-none"
          />
          <span className="text-small font-medium text-content-primary tracking-tight">
            {t("titlebar.app_title")}
          </span>
        </div>
        <div data-tauri-drag-region className="flex-1 h-full" />
        <div className="flex items-stretch h-full">
          <button
            onClick={handleMinimize}
            title={t("titlebar.minimize")}
            className="w-[46px] h-full flex items-center justify-center text-content-secondary hover:text-content-primary hover:bg-white/60 active:bg-white/90 dark:hover:bg-white/10 dark:active:bg-white/20 transition-colors cursor-pointer"
          >
            <Minus className="w-[14px] h-[14px]" />
          </button>
          <button
            onClick={handleToggleMaximize}
            title={isMaximized ? t("titlebar.restore") : t("titlebar.maximize")}
            className="w-[46px] h-full flex items-center justify-center text-content-secondary hover:text-content-primary hover:bg-white/60 active:bg-white/90 dark:hover:bg-white/10 dark:active:bg-white/20 transition-colors cursor-pointer"
          >
            {isMaximized ? (
              <Copy className="w-[12px] h-[12px] rotate-180" />
            ) : (
              <Square className="w-[12px] h-[12px]" />
            )}
          </button>
          <button
            onClick={handleCloseClick}
            title={t("titlebar.close")}
            className="w-[46px] h-full flex items-center justify-center text-content-secondary hover:text-content-inverse hover:bg-status-error active:bg-status-error/90 transition-colors cursor-pointer"
          >
            <X className="w-[15px] h-[15px]" />
          </button>
        </div>
      </header>
      <CloseAppDialog
        isOpen={isCloseDialogOpen}
        onClose={() => setIsCloseDialogOpen(false)}
        onHideToTray={handleHideToTray}
        onExitApp={handleExitApp}
      />
    </>
  );
};

export default TitleBar;