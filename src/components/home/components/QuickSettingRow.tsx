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
          <div className="w-avatar-sm h-avatar-sm flex items-center justify-center shrink-0">
            {icon}
          </div>
          <div className="flex flex-col gap-0.5">
            <span className="text-body font-medium text-content-primary leading-tight">
              {label}
            </span>
            <span className="text-caption text-content-secondary leading-tight">
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

export default QuickSettingRow;
