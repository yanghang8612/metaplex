import * as React from "react";
import tw, { styled } from "twin.macro";

import StyledHeroImage from "./HeroImage";
import BaseButton from "../BaseButton";

// Styled version with macro, usage like in styled components.
const Container = styled.div`
  ${tw`relative sm:max-w-xl text-white text-xl text-left px-10`}
  h1 {
    ${tw`text-4xl font-bold`}
  }
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
const FirstSection = (): React.ReactElement => (
  <StyledHeroImage>
    <Container>
      <h1>Blockchain's Next Generation Conference</h1>
      <RedButton>Request an invite</RedButton>
    </Container>
  </StyledHeroImage>
);

export default FirstSection;
