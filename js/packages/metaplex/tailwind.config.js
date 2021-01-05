module.exports = {
  purge: ["./src/**/**/*.{js,jsx,ts,tsx}"], // purge from all folders
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
    fontFamily: {
      serif: ["maiola", "serif"],
      sans: ["termina", "sans-serif"],
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
