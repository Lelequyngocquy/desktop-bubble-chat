import React from "react";
import { cn } from "../../utils/cn";

export interface OptionCardItem<T extends string> {
  value: T;
  label: string;
  description?: string;
  icon: React.ReactNode;
  iconBoxed?: boolean;
}

interface OptionCardGridProps<T extends string> {
  options: Array<OptionCardItem<T>>;
  value: T;
  onChange: (value: T) => void;
  columns?: 2 | 3;
}

export function OptionCardGrid<T extends string>({
  options,
  value,
  onChange,
  columns = 2,
}: OptionCardGridProps<T>) {
  const gridColsClass = columns === 3 ? "grid-cols-1 sm:grid-cols-3" : "grid-cols-1 sm:grid-cols-2";
  return (
    <div className={cn("grid gap-3 pt-1", gridColsClass)}>
      {options.map((opt) => {
        const isSelected = value === opt.value;
        return (
          <button
            key={opt.value}
            type="button"
            aria-pressed={isSelected}
            onClick={() => onChange(opt.value)}
            className={cn(
              "flex items-center justify-between p-3.5 rounded-control border transition-all text-left cursor-pointer",
              isSelected
                ? "bg-primary/10 border-primary shadow-xs ring-1 ring-primary/20"
                : "bg-background-surface hover:bg-background-subtle border-border hover:border-border",
            )}
          >
            <span className="flex items-center gap-3">
              {opt.iconBoxed ? (
                <span
                  className={cn(
                    "w-8 h-8 rounded-panel flex items-center justify-center shrink-0",
                    isSelected ? "bg-primary/15" : "bg-background-subtle",
                  )}
                >
                  {opt.icon}
                </span>
              ) : (
                opt.icon
              )}
              <span className="flex flex-col">
                <span
                  className={cn(
                    "text-body leading-tight",
                    isSelected ? "font-semibold text-primary" : "font-medium text-content-primary",
                  )}
                >
                  {opt.label}
                </span>
                {opt.description && (
                  <span className="text-caption text-content-muted leading-tight mt-0.5">
                    {opt.description}
                  </span>
                )}
              </span>
            </span>
          </button>
        );
      })}
    </div>
  );
}

export default OptionCardGrid;
