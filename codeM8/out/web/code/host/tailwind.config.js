/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "jit",
  content: [
    "./src/**/*.{html,js,jsx,tsx}",
    "./src/pages/**/*.{html,js}",
    "./src/components/**/*.{html,js,jsx,tsx}",
    "./src/layouts/**/*.{html,js}",
    "./src/index.html",
    "./public/index.html",
  ],
  theme: {
    patterns: {
      opacities: {
        100: "1",
        80: ".80",
        60: ".60",
        40: ".40",
        20: ".20",
        10: ".10",
        5: ".05",
      },
      sizes: {
        1: "0.25rem",
        2: "0.5rem",
        4: "1rem",
        6: "1.5rem",
        8: "2rem",
        16: "4rem",
        20: "5rem",
        24: "6rem",
        32: "8rem",
      },
    },
    extend: {},
  },
  plugins: [require("@tailwindcss/aspect-ratio"), require("tailwindcss-bg-patterns"), require("@tailwindcss/forms")],
};
