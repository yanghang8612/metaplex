module.exports = {
  siteMetadata: {
    title: "Helios Conference",
    description: "Helios - Blockchain’s Next Generation Conference",
    author: `@solana`,
    siteUrl: "http://localhost:8000",
  },
  plugins: [
    "gatsby-plugin-emotion",
    "gatsby-plugin-sharp",
    "gatsby-plugin-react-helmet",
    // "gatsby-plugin-sitemap",
    // "gatsby-plugin-offline",
    {
      resolve: "gatsby-plugin-manifest",
      options: {
        background_color: "#0c0e1b",
        icon: "src/images/icon.png",
      },
    },
    "gatsby-plugin-mdx",
    "gatsby-transformer-sharp",
    {
      resolve: "gatsby-source-filesystem",
      options: {
        name: "images",
        path: "./src/images/",
      },
      __key: "images",
    },
    {
      resolve: "gatsby-source-filesystem",
      options: {
        name: "pages",
        path: "./src/pages/",
      },
      __key: "pages",
    },
    {
      resolve: "gatsby-plugin-react-svg",
      options: {
        rule: {
          include: /\.inline\.svg$/,
        },
      },
    },
    {
      resolve: "gatsby-plugin-postcss",
    },
  ],
};
