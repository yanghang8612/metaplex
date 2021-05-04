module.exports = {
  purge: ["./src/**/**/*.{js,jsx,ts,tsx}"], // purge from all folders
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {
      colors: {
        baseBg: "#121212",
        mainBgStart: "#010101",
        mainBgStop: "#091E21",
        btnPrimaryStart: "#616774",
        btnPrimaryStop: "#403F4C",
        btnBlueStart: "#768BF9",
        btnBlueStop: "#5870EE",
      },
      minHeight: {
        0: "0",
        half: "50%",
        halfScreen: "50vh",
        full: "100%",
      },
    },
    fontFamily: {
      serif: ["maiola", "serif"],
      header: ["Inter", "sans-serif"],
      sans: ["Inter", "sans-serif"],
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
