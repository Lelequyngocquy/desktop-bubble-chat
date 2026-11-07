import React from "react";
import { useTranslation } from "react-i18next";
import { CircleDot, Magnet, Move, Check } from "lucide-react";
import { BubbleSize, BubblePositionMode } from "../../../stores/settingsStore";
import { CommonLargeCard } from "../../common";
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

  const positionOptions: Array<{
    mode: BubblePositionMode;
    label: string;
    desc: string;
    icon: React.ReactNode;
  }> = [
    {
      mode: "auto_tuck",
      label: t("settings:bubble.auto_tuck"),
      desc: t("settings:bubble.auto_tuck_desc"),
      icon: <Magnet className="w-4 h-4 text-primary" />,
    },
    {
      mode: "free",
      label: t("settings:bubble.free"),
      desc: t("settings:bubble.free_desc"),
      icon: <Move className="w-4 h-4 text-emerald-500" />,
    },
  ];

  return (
    <CommonLargeCard
      icon={<CircleDot className="w-4 h-4 text-primary" />}
      title={t("settings:bubble.title")}
    >
      <div className="flex flex-col gap-4">
        {/* Size Selection */}
        <div className="flex flex-col gap-2">
          <span className="text-body font-medium text-content-primary">
            {t("settings:bubble.size_label")}
          </span>
          <BubbleSizePicker
            currentSize={bubbleSize}
            onSelectSize={onSelectSize}
          />
        </div>

        {/* Position Mode Selection */}
        <div className="flex flex-col gap-2 pt-3 border-t border-border">
          <span className="text-body font-medium text-content-primary">
            {t("settings:bubble.position_mode_label")}
          </span>
          <p className="text-caption text-content-secondary m-0">
            {t("settings:bubble.position_mode_desc")}
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
            {positionOptions.map((opt) => {
              const isSelected = bubblePositionMode === opt.mode;
              return (
                <button
                  key={opt.mode}
                  type="button"
                  onClick={() => onSelectPositionMode(opt.mode)}
                  className={`flex items-center justify-between p-3.5 rounded-control border transition-all text-left cursor-pointer ${
                    isSelected
                      ? "bg-primary/10 border-primary shadow-xs ring-1 ring-primary/20"
                      : "bg-background-surface hover:bg-background-subtle border-border hover:border-border"
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-8 h-8 rounded-panel flex items-center justify-center shrink-0 ${
                        isSelected ? "bg-primary/15" : "bg-background-subtle"
                      }`}
                    >
                      {opt.icon}
                    </div>
                    <div className="flex flex-col">
                      <span
                        className={`text-body leading-tight ${
                          isSelected
                            ? "font-semibold text-primary"
                            : "font-medium text-content-primary"
                        }`}
                      >
                        {opt.label}
                      </span>
                      <span className="text-caption text-content-muted leading-tight mt-0.5">
                        {opt.desc}
                      </span>
                    </div>
                  </div>
                  {isSelected && (
                    <div className="w-5 h-5 rounded-full bg-primary text-white flex items-center justify-center shrink-0 ml-2">
                      <Check className="w-3 h-3 stroke-[3]" />
                    </div>
                  )}
                </button>
              );
            })}
          </div>
        </div>
      </div>
    </CommonLargeCard>
  );
};

export default BubbleCustomizationSection;
