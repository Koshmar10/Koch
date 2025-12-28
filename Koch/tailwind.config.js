/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        /* --- Light Theme --- */
        background: "#FAF8F3",
        foreground: "#2D1B0E",
        card: {
          DEFAULT: "#E7DFD5",
          foreground: "#2D1B0E",
        },
        popover: {
          DEFAULT: "#E7DFD5",
          foreground: "#2D1B0E",
        },
        primary: {
          DEFAULT: "#8B6F47",
          foreground: "#FAF8F3",
        },
        secondary: {
          DEFAULT: "#A89379",
          foreground: "#2D1B0E",
        },
        muted: {
          DEFAULT: "#B8A896",
          foreground: "#2D1B0E",
        },
        accent: {
          DEFAULT: "#8B6F47",
          foreground: "#FAF8F3",
        },
        destructive: {
          DEFAULT: "#7A4F3B",
          foreground: "#FAF8F3",
        },
        border: "#D4C4B0",
        input: "#E7DFD5",
        ring: "#8B6F47",

        /* --- Dark Theme --- */
        "background-dark": "#1A1310",
        "foreground-dark": "#F4EEDD",
        "card-dark": {
          DEFAULT: "#2D1F1A",
          foreground: "#F4EEDD",
        },
        "popover-dark": {
          DEFAULT: "#2D1F1A",
          foreground: "#F4EEDD",
        },
        "primary-dark": {
          DEFAULT: "#C9A875",
          foreground: "#1A1310",
        },
        "secondary-dark": {
          DEFAULT: "#6B5747",
          foreground: "#F4EEDD",
        },
        "muted-dark": {
          DEFAULT: "#4A3B30",
          foreground: "#B8A896",
        },
        "accent-dark": {
          DEFAULT: "#C9A875",
          foreground: "#1A1310",
        },
        "destructive-dark": {
          DEFAULT: "#D97757",
          foreground: "#F4EEDD",
        },
        "border-dark": "#3D2F25",
        "input-dark": "#2D1F1A",
        "ring-dark": "#C9A875",

        /* --- Sidebar --- */
        sidebar: {
          DEFAULT: "#F0EBE3",
          foreground: "#2D1B0E",
          primary: "#8B6F47",
          "primary-foreground": "#FAF8F3",
          accent: "#E7DFD5",
          "accent-foreground": "#2D1B0E",
          border: "#D4C4B0",
          ring: "#8B6F47",
          // Dark Sidebar variants
          dark: "#1A1310",
          "foreground-dark": "#F4EEDD",
          "primary-dark": "#C9A875",
          "border-dark": "#3D2F25",
        }
      },
      borderRadius: {
        lg: "0.375rem",
        md: "calc(0.375rem - 2px)",
        sm: "calc(0.375rem - 4px)"
      }
    }
  },
  plugins: []
};