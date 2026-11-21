import React, { ReactNode } from "react";
import { cn } from "../../utils/cn";

interface CommonHintLineProps {
  icon: ReactNode;
  text: ReactNode;
  color?: string;
  rightChildren?: ReactNode;
  className?: string;
}

export const CommonHintLine: React.FC<CommonHintLineProps> = ({
  icon,
  text,
  color = "text-content-secondary",
  rightChildren,
  className = "",
}) => {
  return (
    <div
      className={cn(
        "bg-background-app/60 rounded-panel flex items-center gap-2 text-caption",
        color,
        className,
      )}
    >
      <span
        className={cn("w-3.5 h-3.5 shrink-0 flex items-center justify-center", color)}
      >
        {icon}
      </span>
      <span className="flex-1 leading-snug">{text}</span>
      {rightChildren && <div className="shrink-0 ml-auto">{rightChildren}</div>}
    </div>
  );
};

export default CommonHintLine;
