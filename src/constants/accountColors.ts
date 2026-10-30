/**
 * Bảng 5 màu đại diện độc lập cho tối đa 5 tài khoản (Messenger, Telegram,...)
 * Các màu được lựa chọn kỹ lưỡng để:
 * - Khác biệt hoàn toàn so với các hằng số màu hiện có của ứng dụng (Primary #0866FF, Success #25805C, Warning #D97706, Error #E81123, Grayscale).
 * - Dễ nhận diện thị giác ngay cả ở kích thước nhỏ (Avatar badge, viền bong bóng).
 * - Độ tương phản cao với chữ trắng và nền xanh Messenger.
 */

export interface AccountThemeColor {
  index: number;
  hex: string;
  bgLight: string;
  borderHex: string;
}

export const ACCOUNT_THEME_COLORS: Record<number, AccountThemeColor> = {
  1: {
    index: 1,
    hex: "#06B6D4",
    bgLight: "#ECFEFF",
    borderHex: "#0891B2",
  },
  2: {
    index: 2,
    hex: "#EC4899",
    bgLight: "#FDF2F8",
    borderHex: "#DB2777",
  },
  3: {
    index: 3,
    hex: "#F97316",
    bgLight: "#FFF7ED",
    borderHex: "#EA580C",
  },
  4: {
    index: 4,
    hex: "#8B5CF6",
    bgLight: "#F5F3FF",
    borderHex: "#7C3AED",
  },
  5: {
    index: 5,
    hex: "#10B981",
    bgLight: "#ECFDF5",
    borderHex: "#059669",
  },
};

/**
 * Lấy mã màu đại diện theo số thứ tự tài khoản (1-based index)
 */
export function getAccountThemeColor(index: number = 1): AccountThemeColor {
  const normalized = ((Math.max(1, index) - 1) % 5) + 1;
  return ACCOUNT_THEME_COLORS[normalized] || ACCOUNT_THEME_COLORS[1];
}
