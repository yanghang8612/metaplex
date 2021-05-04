import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";

import SingleSection from "./SingleSection";

/**
 * Multiple Sections from Markdown.
 *
 * @constructor
 */
const MultiSection = (): React.ReactElement => {
  const whichDisplay = useStaticQuery(
    graphql`
      fragment MarkdownSection on MarkdownRemark {
        frontmatter {
          title
          position
          display
          background
          imageWrapper
        }
        rawMarkdownBody
      }
      query {
        nftShop: markdownRemark(frontmatter: { slug: { eq: "nftShop" } }) {
          ...MarkdownSection
        }
        nftExample: markdownRemark(
          frontmatter: { slug: { eq: "nftExample" } }
        ) {
          ...MarkdownSection
        }
        auctions: markdownRemark(frontmatter: { slug: { eq: "auctions" } }) {
          ...MarkdownSection
        }
        collab: markdownRemark(frontmatter: { slug: { eq: "collab" } }) {
          ...MarkdownSection
        }
      }
    `
  );

  const multipleSections = Object.keys(whichDisplay).map((currentDisplay) => {
    const {
      frontmatter: { title, position, display, background, imageWrapper },
      rawMarkdownBody,
    } = whichDisplay[currentDisplay];

    return (
      <SingleSection
        key={currentDisplay}
        name={"nftExample"}
        title={title}
        position={position}
        display={display}
        background={background}
        imageWrapper={imageWrapper}
        rawMarkdownBody={rawMarkdownBody}
      />
    );
  });

  return <>{multipleSections}</>;
};

export default MultiSection;
