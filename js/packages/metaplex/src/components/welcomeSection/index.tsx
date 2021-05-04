import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";
import MockDisplay from "../MockDisplay";

const OuterContainer = styled.div`
  ${tw`
    flex flex-col justify-center items-center
    text-white text-lg
    py-10
    bg-baseBg
  `}
`;

const DropWrapper = tw.div`grid grid-cols-3 gap-5 place-content-center mt-4`;

const MockDrop = tw(MockDisplay)`
  rounded-lg
`;

/**
 * The Welcome Section.
 *
 * @constructor
 */
const WelcomeSection = (): React.ReactElement => {
  const { welcome } = useStaticQuery(
    graphql`
      query {
        welcome: markdownRemark(frontmatter: { slug: { eq: "welcome" } }) {
          frontmatter {
            title
            drop1
            drop2
            drop3
          }
          rawMarkdownBody
        }
      }
    `
  );
  const {
    frontmatter: { title, drop1, drop2, drop3 },
    rawMarkdownBody,
  } = welcome;
  return (
    <>
      <OuterContainer>
        <h3>{title}</h3>
        <p>{rawMarkdownBody}</p>
        <DropWrapper>
          <MockDrop>{drop1}</MockDrop>
          <MockDrop>{drop2}</MockDrop>
          <MockDrop>{drop3}</MockDrop>
        </DropWrapper>
      </OuterContainer>
    </>
  );
};

export default WelcomeSection;
