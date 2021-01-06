import * as React from "react";
import tw from "twin.macro";

import HeliosLogo from "../images/helios_logo.inline.svg";
import BaseButton from "./BaseButton";

const HeaderNavigation = tw.nav`
absolute 
inset-x-0 top-0 
flex flex-row justify-between content-center 
z-10 
p-3
text-white 
bg-transparent
`;

const LogoLink = tw.a`
transition duration-500 
hover:text-indigo-500
p-4
`;

const NavLinkWrapper = tw.div`
p-4 
md:flex flex-row justify-between 
font-sans uppercase
`;

const NavLink = tw.a`
mx-4 my-auto 
text-lg 
hidden
md:block
border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 
transition duration-500
`;

const HeaderButton = tw(BaseButton)`px-8`;

const Header = (): React.ReactElement => {
  return (
    <HeaderNavigation id="nav">
      <LogoLink>
        <HeliosLogo />
      </LogoLink>

      <NavLinkWrapper>
        <NavLink href="#">The Event</NavLink>
        <NavLink href="#partners">Partners</NavLink>
        <NavLink href="#">Press</NavLink>
        <HeaderButton>Attend</HeaderButton>
      </NavLinkWrapper>
    </HeaderNavigation>
  );
};

export default Header;
