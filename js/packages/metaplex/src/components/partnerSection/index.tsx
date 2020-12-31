import * as React from "react";
import tw, { styled } from "twin.macro";

import StyledPartnerSectionBackground from "./PartnerSectionBackground";

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
const PartnerSection = (): React.ReactElement => (
  // @ts-ignore - the id is allowed in BackgroundImage
  <StyledPartnerSectionBackground id="partners">
    <OuterContainer>
      <LeftWrapper />
      <RightContainer>
        <h2>Our Partners</h2>
      </RightContainer>
    </OuterContainer>
  </StyledPartnerSectionBackground>
);

export default PartnerSection;
