import React from "react";
import { cn } from "../../../utils/cn";

export interface SidebarNavItemProps {
  label: string;
  icon: React.ReactNode;
  badge?: React.ReactNode;
  isActive: boolean;
  variant?: "primary" | "secondary";
  onClick: () => void;
}

export const SidebarNavItem: React.FC<SidebarNavItemProps> = ({
  label,
  icon,
  badge,
  isActive,
  variant = "primary",
  onClick,
}) => {
  const isPrimary = variant === "primary";
  return (
    <button
      type="button"
      aria-current={isActive ? "page" : undefined}
      onClick={onClick}
      className={cn(
        "relative flex items-center justify-between px-3 rounded-control transition-all text-left group cursor-pointer",
        isPrimary ? "h-[42px]" : "h-[38px]",
        isActive
          ? "bg-white/70 dark:bg-white/10 shadow-sm"
          : "hover:bg-white/40 dark:hover:bg-white/5 active:bg-white/60 dark:active:bg-white/10",
      )}
    >
      {isActive && (
        <div
          className={cn(
            "absolute left-0 w-[3px] h-[16px] bg-primary rounded-indicator",
            isPrimary ? "top-[13px]" : "top-[11px]",
          )}
        />
      )}
      <div className="flex items-center gap-3">
        <span
          className={cn(
            "transition-colors",
            isActive
              ? "text-primary-foreground"
              : "text-content-secondary group-hover:text-content-primary",
          )}
        >
          {icon}
        </span>
        <span
          className={cn(
            isPrimary ? "text-nav" : "text-body",
            "leading-tight transition-colors",
            isActive
              ? "font-semibold text-primary-foreground"
              : "font-normal text-content-primary group-hover:text-content-primary",
          )}
        >
          {label}
        </span>
      </div>
      {badge !== undefined && (
        <span
          className={cn(
            "text-small font-normal px-1.5 py-0.5 rounded-pill",
            isActive
              ? "text-primary-foreground"
              : "text-content-secondary group-hover:text-content-primary",
          )}
        >
          {badge}
        </span>
      )}
    </button>
  );
};
