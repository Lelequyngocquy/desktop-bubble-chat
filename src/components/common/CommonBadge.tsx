import React, { ReactNode } from "react";

// Định nghĩa các preset màu phổ biến trong app
export type BadgeVariant =
  | "success" // Xanh lá (Đã kết nối, Hoàn thành)
  | "primary" // Xanh dương (Đang hoạt động)
  | "secondary" // Xám (Đang ngủ đông, Chưa kích hoạt)
  | "warning" // Vàng / Cam (Cảnh báo, Đang chờ)
  | "danger"; // Đỏ (Mất kết nối, Lỗi)

const VARIANT_STYLES: Record<
  BadgeVariant,
  { container: string; iconColor?: string }
> = {
  success: {
    container: "bg-[#E9F5EF] text-[#25805C]",
    iconColor: "text-[#25805C]",
  },
  primary: {
    container: "bg-[#EFF6FF] text-[#0866FF]",
    iconColor: "text-[#0866FF]",
  },
  secondary: {
    container: "bg-[#F1F5F9] text-[#64748B]",
    iconColor: "text-[#64748B]",
  },
  warning: {
    container: "bg-[#FFF7ED] text-[#EA580C]",
    iconColor: "text-[#EA580C]",
  },
  danger: {
    container: "bg-[#FEF2F2] text-[#DC2626]",
    iconColor: "text-[#DC2626]",
  },
};

export interface CommonBadgeProps {
  /** Nội dung / trạng thái hiển thị (text hoặc ReactNode) */
  status: ReactNode;
  /** Icon hiển thị bên trái (tùy chọn) */
  icon?: ReactNode;
  /**
   * Chọn theo preset (`success` | `primary` | `secondary` | `warning` | `danger`)
   * hoặc tự truyền chuỗi class CSS tùy biến (ví dụ: "bg-[#EAF2FF] text-[#0866FF]")
   */
  color?: BadgeVariant | string;
  /** Tooltip khi hover (tùy chọn) */
  title?: string;
  /** Bổ sung class tùy biến ngoài nếu cần */
  className?: string;
}

export const CommonBadge: React.FC<CommonBadgeProps> = ({
  status,
  icon,
  color = "secondary",
  title,
  className = "",
}) => {
  // Kiểm tra xem color có nằm trong preset không, nếu không thì xem như custom class
  const isPreset = color in VARIANT_STYLES;
  const colorClasses = isPreset
    ? VARIANT_STYLES[color as BadgeVariant].container
    : color;

  return (
    <span
      title={title}
      className={`inline-flex items-center gap-1 px-2 py-0.5 text-[10.5px] font-medium rounded-full shrink-0 ${colorClasses} ${className}`}
    >
      {icon && (
        <span className="shrink-0 flex items-center [&>svg]:w-3 [&>svg]:h-3">
          {icon}
        </span>
      )}
      {status}
    </span>
  );
};