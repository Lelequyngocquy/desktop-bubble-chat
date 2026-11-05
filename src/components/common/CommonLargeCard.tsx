import React, { ReactNode } from "react";

export interface CommonLargeCardProps {
  icon?: ReactNode;
  title: string;
  subTitle?: ReactNode;
  linkTitle?: ReactNode;
  children?: ReactNode;
  childWidget?: ReactNode;
  className?: string;
}

export const CommonLargeCard: React.FC<CommonLargeCardProps> = ({
  icon,
  title,
  subTitle,
  linkTitle,
  children,
  childWidget,
  className = "",
}) => {
  const content = children ?? childWidget;
  return (
    <div
      className={`bg-background-surface border border-border rounded-card shadow-card p-5 flex flex-col gap-4 ${className}`}
    >
      <div className="flex items-center justify-between pb-2 border-b border-border ">
        <div className="flex items-center gap-2">
          {icon && <div className="shrink-0 flex items-center">{icon}</div>}
          <h2 className="text-body-lg md:text-heading-sm font-semibold text-content-primary m-0">
            {title}
          </h2>
        </div>
        {(subTitle || linkTitle) && (
          <div className="flex items-center gap-3">
            {subTitle &&
              (typeof subTitle === "string" ? (
                <span className="text-caption text-content-secondary">{subTitle}</span>
              ) : (
                subTitle
              ))}
            {linkTitle}
          </div>
        )}
      </div>
      {content}
    </div>
  );
};