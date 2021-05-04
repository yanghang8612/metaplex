import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";
import { GatsbyImage, getImage } from "gatsby-plugin-image";

const EventSectionWrapper = styled.div`
  ${tw`w-full p-4 sm:py-16 block md:flex flex-row`}
  background: linear-gradient(270deg, rgba(97, 103, 116, 0.24) 7.29%, rgba(64, 63, 76, 0.24) 100%);
`;

const Container = styled.div`
  ${tw`flex flex-row justify-center`}
  h5 {
    text-transform: none;
  }
`;

const InnerWrapper = tw.div`
  flex flex-col
  text-white
  w-1/2
  ml-5
`;

const RacImage = tw(GatsbyImage)`
  rounded-full 
`;

/**
 * Cite section.
 *
 * @constructor
 */
const CiteSection = (): React.ReactElement => {
  const { cite, racImage } = useStaticQuery(
    graphql`
      query {
        cite: markdownRemark(frontmatter: { slug: { eq: "cite" } }) {
          frontmatter {
            title
            summary
          }
          rawMarkdownBody
        }
        racImage: file(relativePath: { eq: "rac.png" }) {
          childImageSharp {
            gatsbyImageData(
              quality: 90
              placeholder: BLURRED
              layout: CONSTRAINED
            )
          }
        }
      }
    `
  );

  const racImageData = getImage(racImage);
  return (
    <EventSectionWrapper id="event">
      <Container>
        <RacImage
          image={racImageData}
          alt={`${cite.frontmatter.title} - ${cite.frontmatter.summary}`}
        />
        <InnerWrapper>
          <h3>{cite.rawMarkdownBody}</h3>
          <h4>{cite.frontmatter.title}</h4>
          <h5>{cite.frontmatter.summary}</h5>
        </InnerWrapper>
      </Container>
    </EventSectionWrapper>
  );
};

export default CiteSection;
