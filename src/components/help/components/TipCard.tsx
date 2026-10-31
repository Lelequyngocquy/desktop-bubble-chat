import React from "react";

interface TipCardProps {
  icon: React.ReactNode;
  title: string;
  badge?: string;
  description: string;
}

export const TipCard: React.FC<TipCardProps> = ({
  icon,
  title,
  badge,
  description,
}) => {
  return (
    <div className="p-4 rounded-xl border border-[#E2E7EF] bg-[#F8FAFC] flex items-start gap-3">
      <div className="w-9 h-9 rounded-lg bg-[#EAF2FF] text-[#0759DB] flex items-center justify-center shrink-0">
        {icon}
      </div>
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-2">
          <span className="text-[13px] font-semibold text-[#1E293B]">
            {title}
          </span>
          {badge && (
            <span className="px-1.5 py-0.5 bg-white border border-[#CBD5E1] rounded text-[10px] font-mono text-[#334155] font-semibold">
              {badge}
            </span>
          )}
        </div>
        <span className="text-[11px] text-[#64748B] leading-relaxed">
          {description}
        </span>
      </div>
    </div>
  );
};
