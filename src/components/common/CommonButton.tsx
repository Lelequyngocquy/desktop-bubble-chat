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
  const sizeClasses = {
    sm: "h-8 px-3 text-small gap-1.5",
    md: "h-9 px-3.5 text-body gap-1.5",
    lg: "h-10 px-5 text-nav gap-2",
  }[size];
  const variantClasses = {
    primary:
      "bg-primary hover:bg-primary-foreground active:bg-primary-foreground/90 text-content-inverse shadow-xs border-transparent",
    secondary:
      "bg-background-app hover:bg-border active:bg-border/80 text-content-primary border-transparent",
    outline:
      "bg-background-surface hover:bg-background-subtle active:bg-background-app text-content-primary border border-border",
    ghost:
      "bg-transparent hover:bg-white/50 active:bg-white/80 text-content-primary border-transparent",
    danger:
      "bg-status-error hover:bg-status-error/90 active:bg-status-error/80 text-content-inverse border-transparent",
  }[variant];
  return (
    <button
      type={type}
      title={title}
      disabled={disabled}
      onClick={onClick}
      className={`rounded-control font-medium inline-flex items-center justify-center transition-colors select-none shrink-0 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed ${sizeClasses} ${variantClasses} ${className}`}
    >
      {iconLeading && <span className="shrink-0 flex items-center">{iconLeading}</span>}
      {children && <span>{children}</span>}
      {iconTrailing && <span className="shrink-0 flex items-center">{iconTrailing}</span>}
    </button>
  );
};

export default CommonButton;