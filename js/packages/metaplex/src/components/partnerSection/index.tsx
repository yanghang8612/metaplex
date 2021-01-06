import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import StyledPartnerSectionBackground from "./PartnerSectionBackground";
import PartnerDisplay from "./PartnerDisplay";

const OuterContainer = styled.div`
  ${tw`flex text-white text-xl px-10`}
`;

const LeftWrapper = tw.div`flex-grow`;

const RightContainer = tw.div`w-full sm:w-7/12 2xl:w-8/12`;

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
            main
            investment
            lunar
          }
          rawMarkdownBody
        }
      }
    `
  );
  const {
    frontmatter: { title, ...partnerCat },
    rawMarkdownBody,
  } = partners;
  return (
    // @ts-ignore - the id is allowed in BackgroundImage
    <StyledPartnerSectionBackground id="partners">
      <OuterContainer>
        <LeftWrapper />
        <RightContainer>
          <h3>{title}</h3>
          <p>{rawMarkdownBody}</p>
          <PartnerDisplay partners={partnerCat} />
        </RightContainer>
      </OuterContainer>
    </StyledPartnerSectionBackground>
  );
};

export default PartnerSection;
