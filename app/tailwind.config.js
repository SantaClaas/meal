import materialTailwind from "@claas.dev/material-tailwind";
import formsPlugin from "@tailwindcss/forms";
import plugin from "tailwindcss/plugin";

//TODO migrate to Tailwind CSS 4
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
    // Scroll trigger plugin
    plugin(
      ({ addVariant, addUtilities }) => {
        // The variant makes this usable with all Tailwind CSS classes that you might want to trigger when the parent
        // container is scrolled
        // "true" is not a keyword in CSS so this is just checking for the value any other value than "true" will not trigger this
        addVariant("is-scrolled", "@container style(--is-scrolled: true)");
        // Have to use "animate-..." or it won't include the keyframes
        addUtilities({
          ".animate-scroll-trigger": {
            // animation: "scroll-trigger linear both",
            animationRange: "0 1px",
            animationTimeline: "scroll()",
          },
        });
      },
      {
        theme: {
          extend: {
            // Tailwind CSS seems to not include the keyframes if they are not included in an animation
            animation: {
              "scroll-trigger": "scroll-trigger linear both",
            },
            keyframes: {
              "scroll-trigger": {
                to: {
                  "--is-scrolled": "true",
                },
              },
            },
          },
        },
      }
    ),
  ],
};
