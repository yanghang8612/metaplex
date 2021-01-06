import * as React from "react";
import tw, { styled } from "twin.macro";

const FooterNav = styled.nav`
  ${tw`        
flex flex-row justify-between content-center 
text-white 
bg-transparent
pb-24 pt-2 mx-5 sm:mx-16`}
  border-top: 1px solid rgba(223, 223, 223, 0.5);
`;

const NavLinkWrapper = tw.div`
flex flex-col md:flex-row justify-between 
font-sans uppercase
`;

const LeftLink = tw.a`
text-xs
sm:text-sm
my-auto
font-sans uppercase
transition duration-500 
hover:text-indigo-500
pt-4
max-w-xs
`;

const NavLink = tw.a`
ml-2 sm:ml-4 my-auto
mb-2 sm:mb-auto 
text-xs
sm:text-sm
border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 
transition duration-500
`;

const Footer = (): React.ReactElement => {
  return (
    <FooterNav id="footer-nav">
      <LeftLink href="#">HELIOS BLOCKCHAIN CONFERENCE</LeftLink>

      <NavLinkWrapper>
        <NavLink href="#">About</NavLink>
        <NavLink href="#press">Press</NavLink>
        <NavLink href="#">Get in Touch</NavLink>
        <NavLink href="#partners">Sponsors</NavLink>
      </NavLinkWrapper>
    </FooterNav>
  );
};

export default Footer;
