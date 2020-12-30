import React from "react";
import { useStaticQuery, graphql } from "gatsby";
import { Helmet } from "react-helmet";

interface ISEOProps {
  description?: string;
  lang?: string;
  // meta?: [];
  // keywords?: [];
  title?: string;
}

const SEO = ({
  description,
  lang,
  // meta = [],
  // keywords = [],
  title,
}: ISEOProps): React.ReactElement => {
  const { site } = useStaticQuery(graphql`
    query DefaultSEOQuery {
      site {
        siteMetadata {
          title
          description
          author
        }
      }
    }
  `);

  const metaDescription = description || site.siteMetadata.description;
  // const currentKeywords =
  //   keywords?.length > 0
  //     ? {
  //         name: `keywords`,
  //         content: keywords?.join(`, `),
  //       }
  //     : {};

  return (
    <Helmet
      htmlAttributes={{
        lang,
      }}
      meta={[
        {
          name: `description`,
          content: metaDescription,
        },
        {
          property: `og:title`,
          content: title,
        },
        {
          property: `og:description`,
          content: metaDescription,
        },
        {
          property: `og:type`,
          content: `website`,
        },
        {
          name: `twitter:card`,
          content: `summary`,
        },
        {
          name: `twitter:creator`,
          content: site.siteMetadata.author,
        },
        {
          name: `twitter:title`,
          content: title,
        },
        {
          name: `twitter:description`,
          content: metaDescription,
        },
        // ...meta,
      ]}
      title={title}
      titleTemplate={`%s | ${site.siteMetadata.title}`}
    />
  );
};

SEO.defaultProps = {
  lang: `en`,
  keywords: [],
  meta: [],
};

export default SEO;
