import React from "react";

export interface CommonButtonProps {
  variant?: "primary" | "secondary" | "outline" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  iconLeading?: React.ReactNode;
  iconTrailing?: React.ReactNode;
  children?: React.ReactNode;
  onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void;
  disabled?: boolean;
  className?: string;
  type?: "button" | "submit" | "reset";
  title?: string;
}

export const CommonButton: React.FC<CommonButtonProps> = ({
  variant = "primary",
  size = "md",
  iconLeading,
  iconTrailing,
  children,
  onClick,
  disabled = false,
  className = "",
  type = "button",
  title,
}) => {
  // Kích thước chiều cao và khoảng cách lề
  const sizeClasses = {
    sm: "h-8 px-3 text-[12px] gap-1.5",
    md: "h-9 px-[14px] text-[13px] gap-1.5",
    lg: "h-10 px-5 text-[14px] gap-2",
  }[size];

  // Màu sắc theo biến thể
  const variantClasses = {
    primary:
      "bg-[#0866FF] hover:bg-[#0759DB] active:bg-[#064ec0] text-white shadow-xs border-transparent",
    secondary:
      "bg-[#EEF2F9] hover:bg-[#E2E7EF] active:bg-[#D4DCED] text-[#202733] border-transparent",
    outline:
      "bg-white hover:bg-[#F8FAFD] active:bg-[#EEF2F9] text-[#202733] border border-[#E2E7EF]",
    ghost:
      "bg-transparent hover:bg-white/50 active:bg-white/80 text-[#202733] border-transparent",
    danger:
      "bg-[#E81123] hover:bg-[#BF0F1D] text-white border-transparent",
  }[variant];

  return (
    <button
      type={type}
      title={title}
      disabled={disabled}
      onClick={onClick}
      className={`rounded-[6px] font-medium inline-flex items-center justify-center transition-colors select-none shrink-0 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed ${sizeClasses} ${variantClasses} ${className}`}
    >
      {iconLeading && <span className="shrink-0 flex items-center">{iconLeading}</span>}
      {children && <span>{children}</span>}
      {iconTrailing && <span className="shrink-0 flex items-center">{iconTrailing}</span>}
    </button>
  );
};
