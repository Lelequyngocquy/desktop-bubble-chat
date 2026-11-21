import React from "react";
import { useTranslation } from "react-i18next";
import { CircleDot, Magnet, Move } from "lucide-react";
import { BubbleSize, BubblePositionMode } from "../../../stores/settingsStore";
import { CommonLargeCard, OptionCardGrid, type OptionCardItem } from "../../common";
import { BubbleSizePicker } from "./BubbleSizePicker";

interface BubbleCustomizationSectionProps {
  bubbleSize: BubbleSize;
  onSelectSize: (size: BubbleSize) => void | Promise<void>;
  bubblePositionMode: BubblePositionMode;
  onSelectPositionMode: (mode: BubblePositionMode) => void | Promise<void>;
}

export const BubbleCustomizationSection: React.FC<BubbleCustomizationSectionProps> = ({
  bubbleSize,
  onSelectSize,
  bubblePositionMode,
  onSelectPositionMode,
}) => {
  const { t } = useTranslation(["settings"]);
  const positionOptions: Array<OptionCardItem<BubblePositionMode>> = [
    {
      value: "auto_tuck",
      label: t("settings:bubble.auto_tuck"),
      description: t("settings:bubble.auto_tuck_desc"),
      icon: <Magnet className="w-4 h-4 text-primary" />,
      iconBoxed: true,
    },
    {
      value: "free",
      label: t("settings:bubble.free"),
      description: t("settings:bubble.free_desc"),
      icon: <Move className="w-4 h-4 text-emerald-500" />,
      iconBoxed: true,
    },
  ];
  return (
    <CommonLargeCard
      icon={<CircleDot className="w-4 h-4 text-primary" />}
      title={t("settings:bubble.title")}
    >
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <span className="text-body font-medium text-content-primary">
            {t("settings:bubble.size_label")}
          </span>
          <BubbleSizePicker currentSize={bubbleSize} onSelectSize={onSelectSize} />
        </div>
        <div className="flex flex-col gap-2 pt-3 border-t border-border">
          <span className="text-body font-medium text-content-primary">
            {t("settings:bubble.position_mode_label")}
          </span>
          <p className="text-caption text-content-secondary m-0">
            {t("settings:bubble.position_mode_desc")}
          </p>
          <OptionCardGrid
            options={positionOptions}
            value={bubblePositionMode}
            onChange={onSelectPositionMode}
            columns={2}
          />
        </div>
      </div>
    </CommonLargeCard>
  );
};

export default BubbleCustomizationSection;
