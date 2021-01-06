import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";
import StyledProgrammingSectionBackground from "./ProgrammingBack";
import Divider from "../Divider";

const OuterContainer = styled.div`
  ${tw`text-white px-2 sm:px-5`}
`;

const UpperContainer = styled.div`
  ${tw`w-2/3
    text-white text-xl text-left
    py-3 sm:py-16 
    px-2 sm:px-10`}
  h2 {
    ${tw`mt-2 sm:mt-5 leading-tight`}
  }
`;

const ProgrammingSectionDivider = styled(Divider)`
  ${tw`mx-2 sm:mx-10
    my-2 sm:my-5`}
`;

const BottomContainer = tw.div`
flex
pb-5 sm:pb-10
`;

const RightText = tw.p`
w-full md:w-1/2
px-10
text-xl
`;

const LeftWrapper = tw.div`flex-grow`;

/**
 * Programming section.
 *
 * @constructor
 */
const ProgrammingSection = (): React.ReactElement => {
  const { programming } = useStaticQuery(
    graphql`
      query {
        programming: markdownRemark(frontmatter: { slug: { eq: "prog" } }) {
          frontmatter {
            title
            summary
          }
          rawMarkdownBody
        }
      }
    `
  );
  return (
    <StyledProgrammingSectionBackground>
      <OuterContainer>
        <UpperContainer>
          <h5>{programming.frontmatter.summary}</h5>
          <h2>{programming.frontmatter.title}</h2>
        </UpperContainer>
        <ProgrammingSectionDivider />
        <BottomContainer>
          <LeftWrapper />
          <RightText>{programming.rawMarkdownBody}</RightText>
        </BottomContainer>
      </OuterContainer>
    </StyledProgrammingSectionBackground>
  );
};

export default ProgrammingSection;
