/**
 * DesktopBubbleChat — Common UI constants (Figma Node 3:1145)
 */

export const Color = {
  background: {
    app: "#EEF2F9",       // Nền cửa sổ, sidebar
    workspace: "#F4F7FC", // Nền nội dung chính
    surface: "#FFFFFF",   // Card, nút phụ
    subtle: "#F8FAFD",    // Khu vực phụ bên trong card
  },

  text: {
    primary: "#202733",   // Tiêu đề, nội dung chính
    secondary: "#6B7483", // Mô tả, nhãn phụ
    muted: "#929AA7",     // Thời gian, thông tin ít nổi bật
    inverse: "#FFFFFF",   // Chữ trên nền màu đậm
  },

  primary: {
    main: "#0866FF",      // Nút chính, toggle bật, badge chưa đọc
    foreground: "#0759DB",// Liên kết, chữ trên nền xanh nhạt
    subtle: "#EAF2FF",    // Nền thông tin, avatar mặc định
  },

  border: {
    default: "#E2E7EF",   // Viền card, nút phụ, đường phân cách
    strong: "#929AA7",    // Viền toggle tắt
  },

  status: {
    success: "#25805C",
    successBackground: "#EAF6EF",
  },

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
    control: 6,
    panel: 8,
    card: 12,
    pill: 999,
  },

  layout: {
    sidebarWidth: 232,
    titleBarHeight: 48,
  },
} as const;

export const Typography = {
  fontFamily: "Inter",

  fontSize: {
    badge: 10,
    caption: 11,
    small: 12,
    body: 13,
    navigation: 14,
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
