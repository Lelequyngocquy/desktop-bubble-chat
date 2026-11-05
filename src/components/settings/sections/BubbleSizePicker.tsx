import React from "react";
import { useTranslation } from "react-i18next";
import { BubbleSize } from "../../../stores/settingsStore";

interface BubbleSizePickerProps {
  currentSize: BubbleSize;
  onSelectSize: (size: BubbleSize) => void | Promise<void>;
}

export const BubbleSizePicker: React.FC<BubbleSizePickerProps> = ({
  currentSize,
  onSelectSize,
}) => {
  const { t } = useTranslation(["settings"]);
  const options: Array<{ id: BubbleSize; label: string }> = [
    { id: "sm", label: t("settings:bubble.small") },
    { id: "md", label: t("settings:bubble.medium") },
    { id: "lg", label: t("settings:bubble.large") },
  ];
  return (
    <div className="grid grid-cols-3 gap-3 max-w-dialog-lg">
      {options.map((item) => {
        const isSelected = currentSize === item.id;
        return (
          <button
            key={item.id}
            type="button"
            onClick={() => onSelectSize(item.id)}
            className={`p-3 rounded-card border flex flex-col items-center justify-center text-center gap-1 transition-all cursor-pointer ${
              isSelected
                ? "border-primary bg-primary-subtle text-primary-foreground font-semibold shadow-xs"
                : "border-border bg-background-surface text-content-secondary hover:border-border-strong"
            }`}
          >
            <span className="text-body">{item.label}</span>
          </button>
        );
      })}
    </div>
  );
};

export default BubbleSizePicker;
