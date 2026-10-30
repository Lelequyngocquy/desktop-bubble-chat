import React from "react";

export interface CommonWidgetProps {
  /** Tiêu đề chính của widget */
  title: string;
  /** Badge hiển thị số lượng bên cạnh tiêu đề (tùy chọn) */
  badgeCount?: number;
  /** Dòng mô tả ngắn bên dưới tiêu đề (tùy chọn) */
  description?: string;
  /** Nút hoặc icon góc trên bên phải header */
  headerAction?: React.ReactNode;
  /** Banner cảnh báo hiển thị ngay dưới header (ví dụ Alert cảnh báo) */
  banner?: React.ReactNode;
  /** Nội dung thân widget */
  children: React.ReactNode;
  /** Nội dung góc trái của footer (ví dụ "3/6 cuộc trò chuyện") */
  footerLeft?: React.ReactNode;
  /** Nút hoặc hành động góc phải của footer */
  footerRight?: React.ReactNode;
  /** Class CSS bổ sung cho container ngoài cùng (ví dụ căn chỉnh width trên desktop) */
  className?: string;
  /** Tùy biến class cho vùng body */
  bodyClassName?: string;
}

export const CommonWidget: React.FC<CommonWidgetProps> = ({
  title,
  badgeCount,
  description,
  headerAction,
  banner,
  children,
  footerLeft,
  footerRight,
  className = "",
  bodyClassName = "",
}) => {
  return (
    <div
      className={`bg-white border border-[#E2E7EF] rounded-[12px] shadow-card flex flex-col justify-between min-h-[352px] min-w-0 select-none ${className}`}
    >
      {/* 1. Header */}
      <div className="p-[20px_22px_12px] flex flex-col gap-1.5 border-b border-[#E2E7EF]/50">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h2 className="text-[17px] font-semibold text-[#202733] m-0">
              {title}
            </h2>
            {typeof badgeCount === "number" && (
              <span className="px-[7px] py-[1px] bg-[#EAF2FF] text-[#0759DB] text-[11px] font-semibold rounded-[6px]">
                {badgeCount}
              </span>
            )}
          </div>

          {headerAction && (
            <div className="flex items-center shrink-0">{headerAction}</div>
          )}
        </div>

        {description && (
          <p className="text-[12px] text-[#6B7483] m-0 leading-normal">
            {description}
          </p>
        )}
      </div>

      {/* Banner / Notification (Nếu có) */}
      {banner && <div className="px-[22px] pt-2">{banner}</div>}

      {/* 2. Body Slot */}
      <div className={`px-[22px] flex-1 flex flex-col ${bodyClassName}`}>
        {children}
      </div>

      {/* 3. Footer */}
      {(footerLeft || footerRight) && (
        <div className="h-12 px-[22px] border-t border-[#E2E7EF]/50 flex items-center justify-between shrink-0">
          <div>{footerLeft}</div>
          <div>{footerRight}</div>
        </div>
      )}
    </div>
  );
};
