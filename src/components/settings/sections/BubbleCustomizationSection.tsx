import React from "react";
import { CircleDot } from "lucide-react";
import { BubbleSize } from "../../../stores/settingsStore";
import { BubbleSizePicker } from "./BubbleSizePicker";

interface BubbleCustomizationSectionProps {
  bubbleSize: BubbleSize;
  onSelectSize: (size: BubbleSize) => void | Promise<void>;
}

export const BubbleCustomizationSection: React.FC<BubbleCustomizationSectionProps> = ({
  bubbleSize,
  onSelectSize,
}) => {
  return (
    <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
      <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
        <CircleDot className="w-4 h-4 text-[#0866FF]" />
        <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
          Tùy biến Bong bóng nổi
        </h2>
      </div>

      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <span className="text-[13px] font-medium text-[#1E293B]">
            Kích thước hiển thị
          </span>
          <BubbleSizePicker
            currentSize={bubbleSize}
            onSelectSize={onSelectSize}
          />
        </div>
      </div>
    </div>
  );
};

export default BubbleCustomizationSection;
