import materialTailwind from "@claas.dev/material-tailwind";
import formsPlugin from "@tailwindcss/forms";

/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{js,jsx}", "./index.html"],
  theme: {
    extend: {},
  },
  plugins: [
    formsPlugin,
    materialTailwind({
      source: "#0f172a",
    }),
  ],
};
