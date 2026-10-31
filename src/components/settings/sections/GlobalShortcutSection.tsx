import React from "react";
import { Keyboard } from "lucide-react";
import { SwitchButton } from "../../common";

interface GlobalShortcutSectionProps {
  isGlobalShortcutEnabled: boolean;
  onToggleGlobalShortcut: () => void | Promise<void>;
}

export const GlobalShortcutSection: React.FC<GlobalShortcutSectionProps> = ({
  isGlobalShortcutEnabled,
  onToggleGlobalShortcut,
}) => {
  return (
    <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
      <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
        <Keyboard className="w-4 h-4 text-[#0866FF]" />
        <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
          Phím tắt toàn cục
        </h2>
      </div>

      <div className="flex items-center justify-between gap-4 py-1">
        <div className="flex flex-col gap-0.5">
          <div className="flex items-center gap-2">
            <span className="text-[13px] font-medium text-[#1E293B]">
              Bật/Tắt nhanh bong bóng chat
            </span>
            <span className="px-2 py-0.5 bg-[#F1F5F9] border border-[#CBD5E1] rounded text-[11px] font-mono text-[#334155] font-semibold">
              Ctrl + Shift + M
            </span>
          </div>
          <span className="text-[11px] text-[#64748B]">
            Bấm tổ hợp phím này ở bất cứ đâu trong Windows để ẩn hoặc hiện
            toàn bộ bong bóng nổi
          </span>
        </div>
        <SwitchButton
          checked={isGlobalShortcutEnabled}
          onChange={onToggleGlobalShortcut}
        />
      </div>
    </div>
  );
};

export default GlobalShortcutSection;
