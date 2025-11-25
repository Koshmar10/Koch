/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        mono: ["Roboto Mono", "monospace"],
      },
      colors: {
        primary: "#832103",
        secondary: "#FECDBE",
        dark: "#230901",
        accent: "#9D2804",

        // optional grouped palette
        brand: {
          100: "#FECDBE",
          300: "#9D2804",
          500: "#832103",
          900: "#230901",
        }
      },
    },
  },
  plugins: [],
};
