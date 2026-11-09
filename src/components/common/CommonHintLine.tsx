import React, { ReactNode } from "react";

interface CommonHintLineProps {
  icon: ReactNode;
  text: ReactNode;
  color?: string;
  rightChildren?: ReactNode;
  className?: string;
}

// A single-line hint row: icon + text (same color) + optional right slot pushed to the right.
export const CommonHintLine: React.FC<CommonHintLineProps> = ({
  icon,
  text,
  color = "text-content-secondary",
  rightChildren,
  className = "",
}) => {
  return (
    <div
      className={`  bg-background-app/60 rounded-panel flex items-center gap-2 text-caption ${color} ${className}`}
    >
      <span
        className={`w-3.5 h-3.5 shrink-0 flex items-center justify-center ${color}`}
      >
        {icon}
      </span>
      <span className="flex-1 leading-snug">{text}</span>
      {rightChildren && <div className="shrink-0 ml-auto">{rightChildren}</div>}
    </div>
  );
};

export default CommonHintLine;
