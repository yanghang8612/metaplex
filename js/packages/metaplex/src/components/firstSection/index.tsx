import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import MetaplexSVG from "../../images/metaplex.inline.svg";
import BaseButton from "../BaseButton";

const SectionWrapper = styled.div`
  ${tw`
    min-h-halfScreen w-full
    py-0 inset-0 
    flex flex-col justify-center 
  `}
  &:before,
  &:after {
    ${tw`bg-right-top bg-half sm:bg-right-bottom lg:bg-contain`}
  }
`;

const Container = styled.div`
  ${tw`
    relative 
    sm:max-w-xl md:max-w-3xl
    text-white text-xl text-left 
    px-2 sm:px-12`}
`;

const MetaplexWrapper = styled.div`
  ${tw`
    w-9/12
    mb-20
    `}
`;

const BlueButton = tw(BaseButton)`
  bg-gradient-to-r from-btnBlueStart to-btnBlueStop
  text-sm
  px-8 py-3 
  mt-5
  mr-2
  mb-2
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
    <SectionWrapper>
      <Container>
        <MetaplexWrapper>
          <MetaplexSVG />
        </MetaplexWrapper>
        <h3>{hero.frontmatter.title}</h3>
        <p>{hero.rawMarkdownBody}</p>
        <BlueButton>Request an invite</BlueButton>
        <BaseButton>Request an invite</BaseButton>
      </Container>
    </SectionWrapper>
  );
};

export default FirstSection;
