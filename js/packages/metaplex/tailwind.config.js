module.exports = {
  purge: ["./src/**/**/*.{js,jsx,ts,tsx}"], // purge from all folders
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
    fontFamily: {
      serif: ["maiola", "serif"],
      header: ["orpheuspro", "serif"],
      sans: ["termina", "sans-serif"],
    },
    backgroundSize: {
      auto: "auto",
      cover: "cover",
      contain: "contain",
      half: "50%",
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
