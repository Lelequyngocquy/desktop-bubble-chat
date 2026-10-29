/**
 * Bảng 5 màu đại diện độc lập cho tối đa 5 tài khoản (Messenger, Telegram,...)
 * Các màu được lựa chọn kỹ lưỡng để:
 * - Khác biệt hoàn toàn so với các hằng số màu hiện có của ứng dụng (Primary #0866FF, Success #25805C, Warning #D97706, Error #E81123, Grayscale).
 * - Dễ nhận diện thị giác ngay cả ở kích thước nhỏ (Avatar badge, viền bong bóng).
 * - Độ tương phản cao với chữ trắng và nền xanh Messenger.
 */

export interface AccountThemeColor {
  index: number;
  name: string;
  labelVi: string;
  hex: string;
  bgLight: string;
  borderHex: string;
}

export const ACCOUNT_THEME_COLORS: Record<number, AccountThemeColor> = {
  1: {
    index: 1,
    name: "Electric Violet",
    labelVi: "Tím Thạch Anh",
    hex: "#8B5CF6", // Tím điện tử rực rỡ
    bgLight: "#F5F3FF",
    borderHex: "#7C3AED",
  },
  2: {
    index: 2,
    name: "Sunset Orange",
    labelVi: "Cam Hoàng Hôn",
    hex: "#F97316", // Cam tươi ấm áp
    bgLight: "#FFF7ED",
    borderHex: "#EA580C",
  },
  3: {
    index: 3,
    name: "Electric Aqua",
    labelVi: "Xanh Ngọc Điện",
    hex: "#06B6D4", // Xanh ngọc phát sáng
    bgLight: "#ECFEFF",
    borderHex: "#0891B2",
  },
  4: {
    index: 4,
    name: "Hot Rose",
    labelVi: "Hồng Sen Rực Rỡ",
    hex: "#EC4899", // Hồng sen phong cách
    bgLight: "#FDF2F8",
    borderHex: "#DB2777",
  },
  5: {
    index: 5,
    name: "Emerald Mint",
    labelVi: "Xanh Lục Bảo",
    hex: "#10B981", // Xanh lục ngọc bạc hà
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
