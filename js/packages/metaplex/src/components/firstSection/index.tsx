import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import StyledHeroImage from "./HeroImage";
import BaseButton from "../BaseButton";

// Styled version with macro, usage like in styled components.
const Container = styled.div`
  ${tw`
    relative 
    sm:max-w-xl md:max-w-3xl
    text-white text-xl text-left 
    px-2 sm:px-10`}
`;

const HeroDivider = styled.div`
  ${tw`mt-7 mb-5`}
  height: 0;
  max-width: 60%;
  margin-left: -0.5rem;
  border: 1px solid rgba(223, 223, 223, 0.1);
`;

// Shorthand Version.
const RedButton = tw(BaseButton)`
  hover:bg-red-800 
  text-red-800 hover:text-black 
  border-red-800
  mt-5
`;

/**
 * Top section (Hero section).
 *
 * @constructor
 */
const FirstSection = (): React.ReactElement => {
  const { hero } = useStaticQuery(
    graphql`
      query {
        hero: markdownRemark(frontmatter: { slug: { eq: "hero" } }) {
          frontmatter {
            title
          }
          rawMarkdownBody
        }
      }
    `
  );
  return (
    <StyledHeroImage>
      <Container>
        <h1>{hero.frontmatter.title}</h1>
        <p>{hero.rawMarkdownBody}</p>
        <HeroDivider />
        <RedButton>Request an invite</RedButton>
      </Container>
    </StyledHeroImage>
  );
};

export default FirstSection;
