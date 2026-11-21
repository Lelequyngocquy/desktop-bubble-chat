const withRgb = (cssVar) => `rgb(var(${cssVar}) / <alpha-value>)`;
const withAlphaRgb = (colorVar, alphaVar) =>
  `rgb(var(${colorVar}) / calc(var(${alphaVar}, 1) * <alpha-value>))`;

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: {
          app: withRgb("--color-bg-app"),
          workspace: withRgb("--color-bg-workspace"),
          surface: withRgb("--color-bg-surface"),
          subtle: withRgb("--color-bg-subtle"),
        },
        primary: {
          DEFAULT: withRgb("--color-primary-main"),
          foreground: withRgb("--color-primary-foreground"),
          subtle: withAlphaRgb("--color-primary-subtle", "--alpha-primary-subtle"),
        },
        content: {
          primary: withRgb("--color-text-primary"),
          secondary: withRgb("--color-text-secondary"),
          muted: withRgb("--color-text-muted"),
          inverse: withRgb("--color-text-inverse"),
        },
        border: {
          DEFAULT: withRgb("--color-border-default"),
          strong: withRgb("--color-border-strong"),
          focus: withRgb("--color-border-focus"),
        },
        status: {
          success: withRgb("--color-status-success"),
          successBg: withAlphaRgb("--color-status-success-bg", "--alpha-status-success-bg"),
          successBorder: withAlphaRgb("--color-status-success-border", "--alpha-status-success-border"),
          warning: withRgb("--color-status-warning"),
          warningBg: withAlphaRgb("--color-status-warning-bg", "--alpha-status-warning-bg"),
          warningBorder: withAlphaRgb("--color-status-warning-border", "--alpha-status-warning-border"),
          error: withRgb("--color-status-error"),
          errorBg: withAlphaRgb("--color-status-error-bg", "--alpha-status-error-bg"),
          errorBorder: withAlphaRgb("--color-status-error-border", "--alpha-status-error-border"),
        },
        brand: {
          messenger: "#0866FF",
        },
        accountTheme: {
          1: { main: "#06B6D4", border: "#0891B2", bgLight: withAlphaRgb("--color-acc1-bg", "--alpha-acc1-bg") },
          2: { main: "#EC4899", border: "#DB2777", bgLight: withAlphaRgb("--color-acc2-bg", "--alpha-acc2-bg") },
          3: { main: "#F97316", border: "#EA580C", bgLight: withAlphaRgb("--color-acc3-bg", "--alpha-acc3-bg") },
          4: { main: "#8B5CF6", border: "#7C3AED", bgLight: withAlphaRgb("--color-acc4-bg", "--alpha-acc4-bg") },
          5: { main: "#10B981", border: "#059669", bgLight: withAlphaRgb("--color-acc5-bg", "--alpha-acc5-bg") },
        },
        sidebar: {
          card: withAlphaRgb("--color-sidebar-card", "--alpha-sidebar-card"),
          cardBorder: withAlphaRgb("--color-sidebar-card-border", "--alpha-sidebar-card-border"),
          hover: withAlphaRgb("--color-sidebar-hover", "--alpha-sidebar-hover"),
          active: withAlphaRgb("--color-sidebar-active", "--alpha-sidebar-active"),
        },
        avatar: {
          groupFg: "#7762AC",
          groupBg: "#F2EEFF",
        },
      },
      fontSize: {
        badge: ["10px", { lineHeight: "1.2" }],
        caption: ["11px", { lineHeight: "1.35" }],
        small: ["12px", { lineHeight: "1.4" }],
        body: ["13px", { lineHeight: "1.45" }],
        nav: ["14px", { lineHeight: "1.45" }],
        "body-lg": ["15px", { lineHeight: "1.45" }],
        "heading-sm": ["16px", { lineHeight: "1.4" }],
        section: ["17px", { lineHeight: "1.3" }],
        platform: ["22px", { lineHeight: "1.2" }],
        metric: ["28px", { lineHeight: "1.1" }],
        page: ["30px", { lineHeight: "1.1" }],
      },
      borderRadius: {
        indicator: "2px",
        xs: "4px",
        control: "6px",
        panel: "8px",
        "card-sm": "10px",
        card: "12px",
        modal: "16px",
        pill: "999px",
      },
      width: {
        sidebar: "232px",
        "side-col": "380px",
        "dialog-sm": "400px",
        "dialog-md": "460px",
        "dialog-lg": "580px",
        "dialog-xl": "680px",
        "avatar-sm": "28px",
        "avatar-md": "36px",
        "avatar-lg": "40px",
        "avatar-xl": "48px",
      },
      maxWidth: {
        "dialog-sm": "400px",
        "dialog-md": "460px",
        "dialog-lg": "580px",
        "dialog-xl": "680px",
      },
      height: {
        titlebar: "48px",
        "side-col": "380px",
        "avatar-sm": "28px",
        "avatar-md": "36px",
        "avatar-lg": "40px",
        "avatar-xl": "48px",
      },
      minHeight: {
        "avatar-sm": "28px",
      },
      boxShadow: {
        "2xs": "0 1px 1px 0 rgb(0 0 0 / 0.03)",
        xs: "0 1px 2px 0 rgb(0 0 0 / 0.04)",
        card: "0px 2px 8px 0px rgba(36, 57, 92, 0.02)",
        window: "0px 12px 40px 0px rgba(35, 55, 89, 0.09)",
        bubble: "0px 4px 14px 0px rgba(0, 0, 0, 0.15)",
      },
      fontFamily: {
        sans: [
          "-apple-system",
          "BlinkMacSystemFont",
          '"SF Pro Text"',
          '"SF Pro Display"',
          '"Segoe UI Variable"',
          '"Segoe UI"',
          "Ubuntu",
          "Inter",
          '"Noto Sans"',
          "sans-serif",
        ],
      },
    },
  },
  plugins: [],
};
