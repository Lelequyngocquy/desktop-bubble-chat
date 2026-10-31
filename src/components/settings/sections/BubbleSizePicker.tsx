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
    <div className="grid grid-cols-3 gap-3 max-w-[480px]">
      {BUBBLE_SIZE_OPTIONS.map((item) => {
        const isSelected = currentSize === item.id;
        return (
          <button
            key={item.id}
            type="button"
            onClick={() => onSelectSize(item.id)}
            className={`p-3 rounded-xl border flex flex-col items-center justify-center text-center gap-1 transition-all cursor-pointer ${
              isSelected
                ? "border-[#0866FF] bg-[#EAF2FF] text-[#0759DB] font-semibold shadow-xs"
                : "border-[#E2E7EF] bg-white text-[#475569] hover:border-[#CBD5E1]"
            }`}
          >
            <span className="text-[13px]">{item.label}</span>
            <span className="text-[10px] text-[#64748B] leading-tight">
              {item.desc}
            </span>
          </button>
        );
      })}
    </div>
  );
};

export default BubbleSizePicker;
