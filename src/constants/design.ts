/**
 * DesktopBubbleChat — Single Source of Truth (SSOT) Design System Tokens
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
    borderHex: "#0891B2",
    bgLight: "#ECFEFF",
  },
  2: {
    index: 2,
    hex: "#EC4899",
    borderHex: "#DB2777",
    bgLight: "#FDF2F8",
  },
  3: {
    index: 3,
    hex: "#F97316",
    borderHex: "#EA580C",
    bgLight: "#FFF7ED",
  },
  4: {
    index: 4,
    hex: "#8B5CF6",
    borderHex: "#7C3AED",
    bgLight: "#F5F3FF",
  },
  5: {
    index: 5,
    hex: "#10B981",
    borderHex: "#059669",
    bgLight: "#ECFDF5",
  },
};

/**
 * Lấy mã màu đại diện theo số thứ tự tài khoản (1-based index)
 */
export function getAccountThemeColor(index: number = 1): AccountThemeColor {
  const normalized = ((Math.max(1, index) - 1) % 5) + 1;
  return ACCOUNT_THEME_COLORS[normalized] || ACCOUNT_THEME_COLORS[1];
}

export const Color = {
  background: {
    app: "#EEF2F9",       // Nền tổng thể cửa sổ, sidebar
    workspace: "#F4F7FC", // Nền vùng làm việc nội dung chính
    surface: "#FFFFFF",   // Nền thẻ card, popup dialog, nút nền trắng
    subtle: "#F8FAFD",    // Nền khu vực phụ bên trong card (empty state, preview)
  },

  text: {
    primary: "#202733",   // Tiêu đề chính, nhãn quan trọng (hợp nhất từ #1E293B, #0F172A)
    secondary: "#6B7483", // Mô tả, phụ đề, nhãn phụ (hợp nhất từ #64748B, #475467)
    muted: "#929AA7",     // Thời gian, timestamp, placeholder (hợp nhất từ #94A3B8)
    inverse: "#FFFFFF",   // Chữ trắng trên nền tối/nền màu
  },

  primary: {
    main: "#0866FF",      // Nút chính, toggle bật, badge chưa đọc (hợp nhất từ #0084FF, #0068FF)
    foreground: "#0759DB",// Liên kết, chữ xanh trên nền nhạt, hover
    subtle: "#EAF2FF",    // Nền badge, nền icon nổi bật (hợp nhất từ #EFF6FF, #F0F4FF, #F0F7FF)
  },

  border: {
    default: "#E2E7EF",   // Viền card, nút phụ, gạch phân cách (hợp nhất từ #CBD5E1, #E2E8F0)
    strong: "#929AA7",    // Viền toggle tắt, viền phân cách đậm
    focus: "#0866FF",     // Viền khi focus ô input
  },

  status: {
    success: "#25805C",             // Trạng thái đã kết nối, hoàn thành
    successBackground: "#EAF6EF",   // Nền badge thành công (hợp nhất từ #E9F5EF)
    successBorder: "#BBF7D0",       // Viền thông báo thành công

    warning: "#D97706",             // Cảnh báo, đang tải, chưa đăng nhập
    warningBackground: "#FFF3E8",   // Nền badge cảnh báo
    warningBorder: "#FED7AA",       // Viền thông báo cảnh báo

    error: "#DC2626",               // Báo lỗi, xóa tài khoản, nguy hiểm (hợp nhất từ #E81123)
    errorBackground: "#FEF2F2",     // Nền badge lỗi (hợp nhất từ #FFF5F5)
    errorBorder: "#FECACA",         // Viền nút danger
  },

  brand: {                          // Màu thương hiệu các nền tảng
    messenger: "#0866FF",
    telegram: "#229ED9",
    zalo: "#0068FF",
    discord: "#5865F2",
  },

  accountTheme: ACCOUNT_THEME_COLORS,

  avatar: {
    groupForeground: "#7762AC",
    groupBackground: "#F2EEFF",
  },

  overlay: {
    navigationSelected: "rgba(255, 255, 255, 0.69)",
    backgroundService: "rgba(255, 255, 255, 0.50)",
    upcomingPlatform: "rgba(255, 255, 255, 0.65)",
  },
} as const;

export const Dimen = {
  spacing: {
    none: 0,
    xxs: 2,
    xs: 4,
    sm: 8,
    md: 12,
    lg: 16,
    xl: 24,
    xxl: 32,
  },

  padding: {
    toggle: 4,
    navigationX: 12,
    buttonX: 14,
    platformCard: 16,
    summaryCard: 18,
    sectionTop: 20,
    sectionX: 22,
    connectionCard: 24,
    workspaceX: 32,
    workspaceTop: 28,
    workspaceBottom: 20,
  },

  borderRadius: {
    indicator: 2,
    xs: 4,
    control: 6,
    panel: 8,
    cardSm: 10,
    card: 12,
    modal: 16,
    pill: 999,
  },

  layout: {
    sidebarWidth: 232,
    titleBarHeight: 48,
    sideColumnWidth: 380,
    dialogSmall: 400,
    dialogMedium: 460,
    dialogLarge: 580,
    dialogXLarge: 680,
    avatarSm: 28,
    avatarMd: 36,
    avatarLg: 40,
    avatarXl: 48,
  },
} as const;

export const Typography = {
  fontFamily: "Inter, system-ui, -apple-system, sans-serif",

  fontSize: {
    badge: 10,
    caption: 11,
    small: 12,
    body: 13,
    navigation: 14,
    bodyLg: 15,
    headingSm: 16,
    sectionTitle: 17,
    platformTitle: 22,
    metric: 28,
    pageTitle: 30,
  },

  fontWeight: {
    regular: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },

  lineHeight: {
    default: 1.45,
    description: 1.4,
  },
} as const;
