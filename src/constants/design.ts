export interface AccountThemeColor {
  index: number;
  hex: string;
  borderHex: string;
  bgLight: string;
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
 * Returns theme color by 1-based account index
 */
export function getAccountThemeColor(index: number = 1): AccountThemeColor {
  const normalized = ((Math.max(1, index) - 1) % 5) + 1;
  return ACCOUNT_THEME_COLORS[normalized] || ACCOUNT_THEME_COLORS[1];
}

export const Color = {
  background: {
    app: "#EEF2F9",       // Window and sidebar background
    workspace: "#F4F7FC", // Main workspace background
    surface: "#FFFFFF",   // Card and modal background
    subtle: "#F8FAFD",    // Secondary background within cards
  },

  text: {
    primary: "#202733",   // Primary headers and labels
    secondary: "#6B7483", // Descriptions and subtitles
    muted: "#929AA7",     // Timestamps and placeholders
    inverse: "#FFFFFF",   // White text on dark/colored background
  },

  primary: {
    main: "#0866FF",      // Primary action and active switches
    foreground: "#0759DB",// Links and blue text on light background
    subtle: "#EAF2FF",    // Badge background
  },

  border: {
    default: "#E2E7EF",   // Default borders
    strong: "#929AA7",    // Inactive switches and strong borders
    focus: "#0866FF",     // Focus outlines
  },

  status: {
    success: "#25805C",             // Connected/Success status
    successBackground: "#EAF6EF",   // Success badge background
    successBorder: "#BBF7D0",       // Success border

    warning: "#D97706",             // Warning/Pending status
    warningBackground: "#FFF3E8",   // Warning badge background
    warningBorder: "#FED7AA",       // Warning border

    error: "#DC2626",               // Error status
    errorBackground: "#FEF2F2",     // Error badge background
    errorBorder: "#FECACA",         // Danger button border
  },

  brand: {
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