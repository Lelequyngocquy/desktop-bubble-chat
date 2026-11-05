import React, { ReactNode } from "react";

export type BadgeVariant =
  | "success" // Green (Connected, Done)
  | "primary" // Blue (Active)
  | "secondary" // Gray (Hibernating, Inactive)
  | "warning" // Yellow / Orange (Warning, Pending)
  | "danger"; // Red (Disconnected, Error)

const VARIANT_STYLES: Record<
  BadgeVariant,
  { container: string; iconColor?: string }
> = {
  success: {
    container: "bg-status-successBg text-status-success",
    iconColor: "text-status-success",
  },
  primary: {
    container: "bg-primary-subtle text-primary",
    iconColor: "text-primary",
  },
  secondary: {
    container: "bg-background-app text-content-secondary",
    iconColor: "text-content-secondary",
  },
  warning: {
    container: "bg-status-warningBg text-status-warning",
    iconColor: "text-status-warning",
  },
  danger: {
    container: "bg-status-errorBg text-status-error",
    iconColor: "text-status-error",
  },
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
  const isPreset = color in VARIANT_STYLES;
  const colorClasses = isPreset
    ? VARIANT_STYLES[color as BadgeVariant].container
    : color;
  return (
    <span
      title={title}
      className={`inline-flex items-center gap-1 px-2 py-0.5 text-caption font-medium rounded-pill shrink-0 ${colorClasses} ${className}`}
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