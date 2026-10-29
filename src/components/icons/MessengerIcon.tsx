import React from "react";

interface IconProps {
  className?: string;
  size?: number;
}

export const MessengerIcon: React.FC<IconProps> = ({ className = "w-6 h-6", size }) => {
  return (
    <svg
      viewBox="0 0 36 36"
      width={size}
      height={size}
      className={className}
      fill="currentColor"
    >
      <path d="M18 1C8.61 1 1 8.16 1 17c0 5.04 2.5 9.54 6.43 12.45V35l5.29-2.91c1.68.47 3.46.72 5.28.72 9.39 0 17-7.16 17-16S27.39 1 18 1zm1.75 21.62l-4.34-4.63-8.48 4.63 9.33-9.91 4.45 4.63 8.37-4.63-9.33 9.91z" />
    </svg>
  );
};
