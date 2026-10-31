import React from "react";
import { SwitchButton } from "../../common";

interface QuickSettingRowProps {
  label: string;
  description: string;
  icon: React.ReactNode;
  value: boolean;
  disabled?: boolean;
  onChange: () => void | Promise<void>;
}

export const QuickSettingRow: React.FC<QuickSettingRowProps> = ({
  label,
  description,
  icon,
  value,
  disabled = false,
  onChange,
}) => {
  return (
    <div className="flex flex-col">
      <div
        className={`h-[56px] flex items-center justify-between gap-3 group ${
          disabled ? "opacity-60" : ""
        }`}
      >
        <div className="flex items-center gap-3">
          <div className="w-7 h-7 flex items-center justify-center shrink-0">
            {icon}
          </div>
          <div className="flex flex-col gap-0.5">
            <span className="text-[13px] font-medium text-[#202733] leading-tight">
              {label}
            </span>
            <span className="text-[11px] text-[#6B7483] leading-tight">
              {description}
            </span>
          </div>
        </div>

        <SwitchButton
          checked={value}
          onChange={onChange}
          disabled={disabled}
          aria-label={label}
        />
      </div>
    </div>
  );
};
