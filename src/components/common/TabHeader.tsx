import React from "react";

interface TabHeaderProps {
  eyebrow?: string;
  title: string;
  description?: string;
  rightSlot?: React.ReactNode;
}

export const TabHeader: React.FC<TabHeaderProps> = ({
  eyebrow,
  title,
  description,
  rightSlot,
}) => {
  return (
    <header className="shrink-0 bg-[#F4F7FC] px-8 pt-5 pb-4 border-b border-[#E2E7EF] flex items-center justify-between gap-4 select-none">
      <div className="flex flex-col gap-0.5">
        {eyebrow && (
          <span className="text-[11px] font-semibold text-[#6B7483] uppercase tracking-wider leading-none">
            {eyebrow}
          </span>
        )}
        <div className="flex items-center gap-3">
          <h1 className="text-[22px] font-bold text-[#202733] tracking-tight leading-tight m-0">
            {title}
          </h1>
        </div>
        {description && (
          <p className="text-[13px] text-[#6B7483] m-0 leading-normal">
            {description}
          </p>
        )}
      </div>

      {rightSlot && <div className="shrink-0 flex items-center">{rightSlot}</div>}
    </header>
  );
};
