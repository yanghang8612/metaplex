module.exports = {
  siteMetadata: {
    title: 'METAPLEX',
    description: 'Launch an NFT marketplace on Solana',
    author: `@solana`,
    siteUrl: 'http://localhost:8000',
  },
  plugins: [
    'gatsby-plugin-emotion',
    'gatsby-plugin-image',
    'gatsby-plugin-sharp',
    'gatsby-plugin-react-helmet',
    // "gatsby-plugin-sitemap",
    // "gatsby-plugin-offline",
    {
      resolve: 'gatsby-plugin-manifest',
      options: {
        background_color: '#000000',
        icon: 'src/images/icon.png',
      },
    },
    // "gatsby-plugin-mdx",
    'gatsby-transformer-sharp',
    {
      resolve: 'gatsby-source-filesystem',
      options: {
        name: 'images',
        path: `${__dirname}/src/images/`,
      },
      __key: 'images',
    },
    {
      resolve: 'gatsby-source-filesystem',
      options: {
        name: 'pages',
        path: `${__dirname}/src/pages/`,
      },
      __key: 'pages',
    },
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `markdown`,
        path: `${__dirname}/src/markdown`,
      },
      __key: 'markdown',
    },
    'gatsby-transformer-remark',
    {
      resolve: 'gatsby-plugin-react-svg',
      options: {
        rule: {
          include: /\.inline\.svg$/,
        },
      },
    },
    {
      resolve: 'gatsby-plugin-postcss',
    },
  ],
};
