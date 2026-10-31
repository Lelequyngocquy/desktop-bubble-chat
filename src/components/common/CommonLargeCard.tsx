import React, { ReactNode } from "react";

export interface CommonLargeCardProps {
  /** Icon hiển thị ở header (nhận icon trực tiếp hoặc wrapper chứa icon) */
  icon?: ReactNode;
  /** Tiêu đề chính của thẻ */
  title: string;
  /** Subtitle phụ nằm bên phải title */
  subTitle?: ReactNode;
  /** Link hoặc nút hành động ở góc phải (nhận ReactNode hoặc chuỗi/nút bấm) */
  linkTitle?: ReactNode;
  /** Nội dung con bên dưới header (có thể dùng children hoặc childWidget) */
  children?: ReactNode;
  childWidget?: ReactNode;
  /** Class tùy biến cho khung ngoài (ví dụ: w-full xl:w-[380px] shrink-0) */
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
      className={`bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4 ${className}`}
    >
      {/* Header luôn cố định có đường viền gạch chân border-b */}
      <div className="flex items-center justify-between pb-2 border-b border-[#E2E7EF]/60">
        {/* Khối bên trái: Icon + Tiêu đề */}
        <div className="flex items-center gap-2">
          {icon && <div className="shrink-0 flex items-center">{icon}</div>}
          <h2 className="text-[15px] md:text-[16px] font-semibold text-[#1E293B] m-0">
            {title}
          </h2>
        </div>

        {/* Khối bên phải: subTitle nằm trước (bên trái) linkTitle nếu có cả hai */}
        {(subTitle || linkTitle) && (
          <div className="flex items-center gap-3">
            {subTitle &&
              (typeof subTitle === "string" ? (
                <span className="text-[11px] text-[#64748B]">{subTitle}</span>
              ) : (
                subTitle
              ))}
            {linkTitle}
          </div>
        )}
      </div>

      {/* Nội dung bên dưới */}
      {content}
    </div>
  );
};
