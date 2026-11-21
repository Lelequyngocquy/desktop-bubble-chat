import React, { ReactNode } from "react";
import { cn } from "../../utils/cn";

export type BadgeVariant =
  | "success"
  | "primary"
  | "secondary"
  | "warning"
  | "danger";

const VARIANT_STYLES: Record<BadgeVariant, string> = {
  success: "bg-status-successBg text-status-success",
  primary: "bg-primary-subtle text-primary",
  secondary: "bg-background-subtle border border-border text-content-secondary",
  warning: "bg-status-warningBg text-status-warning",
  danger: "bg-status-errorBg text-status-error",
};

export interface CommonBadgeProps {
  status: ReactNode;
  icon?: ReactNode;
  color?: BadgeVariant | string;
  title?: string;
  className?: string;
}

export const CommonBadge: React.FC<CommonBadgeProps> = ({
  status,
  icon,
  color = "secondary",
  title,
  className = "",
}) => {
  const colorClasses = color in VARIANT_STYLES ? VARIANT_STYLES[color as BadgeVariant] : color;
  return (
    <span
      title={title}
      className={cn(
        "inline-flex items-center gap-1 px-2 py-0.5 text-caption font-medium rounded-pill shrink-0",
        colorClasses,
        className,
      )}
    >
      {icon && (
        <span className="shrink-0 flex items-center [&>svg]:w-3 [&>svg]:h-3">
          {icon}
        </span>
      )}
      {status}
    </span>
  );
};

export default CommonBadge;
