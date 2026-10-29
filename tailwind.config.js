/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: {
          app: "#EEF2F9",
          workspace: "#F4F7FC",
          surface: "#FFFFFF",
          subtle: "#F8FAFD",
        },
        primary: {
          DEFAULT: "#0866FF",
          main: "#0866FF",
          foreground: "#0759DB",
          subtle: "#EAF2FF",
        },
        content: {
          primary: "#202733",
          secondary: "#6B7483",
          muted: "#929AA7",
          inverse: "#FFFFFF",
        },
        border: {
          DEFAULT: "#E2E7EF",
          strong: "#929AA7",
        },
        status: {
          success: "#25805C",
          successBg: "#EAF6EF",
        },
        avatar: {
          groupFg: "#7762AC",
          groupBg: "#F2EEFF",
        },
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "-apple-system", "BlinkMacSystemFont", "Segoe UI", "Roboto", "sans-serif"],
      },
      boxShadow: {
        card: "0px 2px 8px 0px rgba(36, 57, 92, 0.02)",
        window: "0px 12px 40px 0px rgba(35, 55, 89, 0.09)",
        bubble: "0px 4px 14px 0px rgba(0, 0, 0, 0.15)",
      },
      borderRadius: {
        indicator: "2px",
        control: "6px",
        panel: "8px",
        card: "12px",
        pill: "999px",
      },
    },
  },
  plugins: [],
};
