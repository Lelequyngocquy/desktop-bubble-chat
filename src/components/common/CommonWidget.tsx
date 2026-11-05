import React from "react";

export interface CommonWidgetProps {
  title: string;
  badgeCount?: number;
  description?: string;
  headerAction?: React.ReactNode;
  banner?: React.ReactNode;
  children: React.ReactNode;
  footerLeft?: React.ReactNode;
  footerRight?: React.ReactNode;
  className?: string;
  bodyClassName?: string;
}

export const CommonWidget: React.FC<CommonWidgetProps> = ({
  title,
  badgeCount,
  description,
  headerAction,
  banner,
  children,
  footerLeft,
  footerRight,
  className = "",
  bodyClassName = "",
}) => {
  return (
    <div
      className={`bg-background-surface border border-border rounded-card shadow-card flex flex-col justify-between min-w-0 select-none ${className}`}
    >
      <div className="p-[20px_22px_12px] flex flex-col gap-1.5 border-b border-border shrink-0">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h2 className="text-section font-semibold text-content-primary m-0">
              {title}
            </h2>
            {typeof badgeCount === "number" && (
              <span className="px-[7px] py-[1px] bg-primary-subtle text-primary-foreground text-caption font-semibold rounded-control">
                {badgeCount}
              </span>
            )}
          </div>
          {headerAction && (
            <div className="flex items-center shrink-0">{headerAction}</div>
          )}
        </div>
        {description && (
          <p className="text-small text-content-secondary m-0 leading-normal">
            {description}
          </p>
        )}
      </div>
      {banner && <div className="px-[22px] pt-2 shrink-0">{banner}</div>}
      <div className={`px-[22px] flex-1 min-h-0 flex flex-col ${bodyClassName}`}>
        {children}
      </div>
      {(footerLeft || footerRight) && (
        <div className="h-12 px-[22px] border-t border-border flex items-center justify-between shrink-0">
          <div>{footerLeft}</div>
          <div>{footerRight}</div>
        </div>
      )}
    </div>
  );
};

export default CommonWidget;