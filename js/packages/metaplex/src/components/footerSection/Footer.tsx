import * as React from "react";
import tw, { styled } from "twin.macro";

import SolanaLogo from "../../images/dark-mark-white.inline.svg";
import SolanaText from "../../images/dark-horizontal-white.inline.svg";

// @ts-ignore
const StyledSolanaLogo = styled(SolanaLogo)`
  margin: auto;
  height: 26px;
`;

// @ts-ignore
const StyledSolanaText = styled(SolanaText)`
  margin: auto;
  height: 33px;
`;

const FooterNavigation = tw.nav`
flex flex-row justify-between content-center 
text-white 
bg-transparent
mx-5 sm:mx-12
`;

const LogoLink = tw.a`
flex flex-row justify-between 
transition duration-500 
hover:text-indigo-500
`;

const NavLinkWrapper = tw.div`
flex flex-col md:flex-row justify-between 
font-sans uppercase
`;

const NavLink = tw.a`
mx-4 my-auto 
text-sm 
mb-2 sm:mb-auto 
text-xs
sm:text-sm
border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 
transition duration-500
`;

const Footer = (): React.ReactElement => {
  return (
    <FooterNavigation id="footer">
      <LogoLink href="https://solana.com">
        <h6 className="hidden sm:block">Produced by</h6>
        <StyledSolanaLogo />
        <StyledSolanaText />
      </LogoLink>

      <NavLinkWrapper>
        <NavLink href="#">Terms of use</NavLink>
        <NavLink href="#">Privacy Policy</NavLink>
      </NavLinkWrapper>
    </FooterNavigation>
  );
};

export default Footer;
