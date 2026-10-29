import React from "react";

export interface SwitchButtonProps {
  checked: boolean;
  onChange?: (checked: boolean) => void;
  disabled?: boolean;
  size?: "sm" | "md";
  className?: string;
  ariaLabel?: string;
}

export const SwitchButton: React.FC<SwitchButtonProps> = ({
  checked,
  onChange,
  disabled = false,
  size = "md",
  className = "",
  ariaLabel,
}) => {
  const isSm = size === "sm";
  const trackSize = isSm ? "w-[30px] h-4" : "w-[38px] h-5";
  const thumbSize = isSm ? "w-3 h-3" : "w-3.5 h-3.5";
  const translateClass = isSm
    ? checked
      ? "translate-x-[14px]"
      : "translate-x-0"
    : checked
    ? "translate-x-[18px]"
    : "translate-x-0";

  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      disabled={disabled}
      onClick={() => !disabled && onChange?.(!checked)}
      className={`rounded-full p-[2px] transition-colors relative shrink-0 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed ${trackSize} ${
        checked
          ? "bg-[#0866FF]"
          : "bg-transparent border border-[#929AA7]"
      } ${className}`}
    >
      <div
        className={`rounded-full transition-transform ${thumbSize} ${translateClass} ${
          checked ? "bg-white" : "bg-[#6B7483]"
        }`}
      />
    </button>
  );
};
