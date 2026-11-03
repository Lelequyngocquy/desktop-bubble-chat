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
    <div className="p-4 rounded-card border border-border bg-background-subtle flex items-start gap-3">
      <div className="w-avatar-md h-avatar-md rounded-panel bg-primary-subtle text-primary-foreground flex items-center justify-center shrink-0">
        {icon}
      </div>
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-2">
          <span className="text-body font-semibold text-content-primary">
            {title}
          </span>
          {badge && (
            <span className="px-1.5 py-0.5 bg-background-surface border border-border rounded-xs text-badge font-mono text-content-primary font-semibold">
              {badge}
            </span>
          )}
        </div>
        <span className="text-caption text-content-secondary leading-relaxed">
          {description}
        </span>
      </div>
    </div>
  );
};

export default TipCard;
