import * as React from "react";
import tw, { styled } from "twin.macro";

import StyledPartnerSectionBackground from "./PartnerSectionBackground";
import { graphql, useStaticQuery } from "gatsby";

const OuterContainer = styled.div`
  ${tw`flex text-white text-xl px-10 object-right`}
  h2 {
    ${tw`text-4xl font-bold`}
  }
`;

const LeftWrapper = tw.div`flex-grow`;

const RightContainer = tw.div`w-9/12`;

/**
 * The Partner Section.
 *
 * @constructor
 */
const PartnerSection = (): React.ReactElement => {
  const { partners } = useStaticQuery(
    graphql`
      query {
        partners: markdownRemark(frontmatter: { slug: { eq: "partners" } }) {
          frontmatter {
            title
          }
          rawMarkdownBody
        }
      }
    `
  );
  return (
    // @ts-ignore - the id is allowed in BackgroundImage
    <StyledPartnerSectionBackground id="partners">
      <OuterContainer>
        <LeftWrapper />
        <RightContainer>
          <h2>{partners.frontmatter.title}</h2>
          <p>{partners.rawMarkdownBody}</p>
        </RightContainer>
      </OuterContainer>
    </StyledPartnerSectionBackground>
  );
};

export default PartnerSection;
