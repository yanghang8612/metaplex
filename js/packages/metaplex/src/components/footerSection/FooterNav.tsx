import * as React from "react";
import tw, { styled } from "twin.macro";

const FooterNav = styled.nav`
  ${tw`        
flex flex-row justify-between content-center 
text-white 
bg-transparent
pb-10 pt-2 mx-8`}
  border-top: 1px solid rgba(223, 223, 223, 0.5);
`;

const NavLinkWrapper = tw.div`
md:flex flex-row justify-between 
font-sans uppercase
`;

const LeftLink = tw.a`
text-sm
font-sans uppercase
transition duration-500 
hover:text-indigo-500
pt-4
max-w-xs
`;

const NavLink = tw.a`
ml-4 my-auto 
text-sm 
hidden
md:block
border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 
transition duration-500
`;

const Footer = (): React.ReactElement => {
  return (
    <FooterNav id="footer-nav">
      <LeftLink href="#">HELIOS BLOCKCHAIN CONFERENCE</LeftLink>

      <NavLinkWrapper>
        <NavLink href="#">About</NavLink>
        <NavLink href="#">Press</NavLink>
        <NavLink href="#">Get in Touch</NavLink>
        <NavLink href="#">Sponsors</NavLink>
      </NavLinkWrapper>
    </FooterNav>
  );
};

export default Footer;
