import React from "react";
import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation(["settings"]);
  return (
    <CommonLargeCard
      icon={<CircleDot className="w-4 h-4 text-primary" />}
      title={t("settings:bubble.title")}
    >
      <div className="flex flex-col gap-2">
        <span className="text-body font-medium text-content-primary">
          {t("settings:bubble.size_label")}
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
