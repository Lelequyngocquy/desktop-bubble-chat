import React from "react";

interface TabHeaderProps {
  eyebrow?: string;
  title: string;
  description?: string;
  rightSlot?: React.ReactNode;
}

export const TabHeader: React.FC<TabHeaderProps> = ({
  title,
  description,
  rightSlot,
}) => {
  return (
    <header className="shrink-0 bg-background-workspace px-8 pt-5 pb-4 border-b border-border flex items-center justify-between gap-4 select-none">
      <div className="flex flex-col gap-0.5">
        <div className="flex items-center gap-3">
          <h1 className="text-platform font-bold text-content-primary tracking-tight leading-tight m-0">
            {title}
          </h1>
        </div>
        {description && (
          <p className="text-body text-content-secondary m-0 leading-normal">
            {description}
          </p>
        )}
      </div>

      {rightSlot && (
        <div className="shrink-0 flex items-center">{rightSlot}</div>
      )}
    </header>
  );
};

export default TabHeader;
