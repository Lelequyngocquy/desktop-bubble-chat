import React from "react";
import { Keyboard } from "lucide-react";
import { SwitchButton, CommonLargeCard } from "../../common";

interface GlobalShortcutSectionProps {
  isGlobalShortcutEnabled: boolean;
  onToggleGlobalShortcut: () => void | Promise<void>;
}

export const GlobalShortcutSection: React.FC<GlobalShortcutSectionProps> = ({
  isGlobalShortcutEnabled,
  onToggleGlobalShortcut,
}) => {
  return (
    <CommonLargeCard
      icon={<Keyboard className="w-4 h-4 text-[#0866FF]" />}
      title="Phím tắt toàn cục"
    >
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
    </CommonLargeCard>
  );
};

export default GlobalShortcutSection;
