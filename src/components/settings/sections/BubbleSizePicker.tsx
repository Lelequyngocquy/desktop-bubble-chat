import React from "react";
import { BubbleSize } from "../../../stores/settingsStore";

interface BubbleSizeOption {
  id: BubbleSize;
  label: string;
  desc: string;
}

export const BUBBLE_SIZE_OPTIONS: BubbleSizeOption[] = [
  {
    id: "sm",
    label: "Nhỏ (48px)",
    desc: "Gọn gàng, ít choán màn hình",
  },
  {
    id: "md",
    label: "Vừa (56px)",
    desc: "Kích thước chuẩn khuyến nghị",
  },
  {
    id: "lg",
    label: "Lớn (64px)",
    desc: "Dễ click, avatar to rõ",
  },
];

interface BubbleSizePickerProps {
  currentSize: BubbleSize;
  onSelectSize: (size: BubbleSize) => void | Promise<void>;
}

export const BubbleSizePicker: React.FC<BubbleSizePickerProps> = ({
  currentSize,
  onSelectSize,
}) => {
  return (
    <div className="grid grid-cols-3 gap-3 max-w-dialog-lg">
      {BUBBLE_SIZE_OPTIONS.map((item) => {
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
            <span className="text-badge text-content-secondary leading-tight">
              {item.desc}
            </span>
          </button>
        );
      })}
    </div>
  );
};

export default BubbleSizePicker;
