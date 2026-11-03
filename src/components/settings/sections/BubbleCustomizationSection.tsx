import React from "react";
import { CircleDot } from "lucide-react";
import { BubbleSize } from "../../../stores/settingsStore";
import { CommonLargeCard } from "../../common";
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
    <CommonLargeCard
      icon={<CircleDot className="w-4 h-4 text-primary" />}
      title="Tùy biến Bong bóng nổi"
    >
      <div className="flex flex-col gap-2">
        <span className="text-body font-medium text-content-primary">
          Kích thước hiển thị
        </span>
        <BubbleSizePicker
          currentSize={bubbleSize}
          onSelectSize={onSelectSize}
        />
      </div>
    </CommonLargeCard>
  );
};

export default BubbleCustomizationSection;
