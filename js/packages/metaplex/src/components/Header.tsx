import * as React from "react";
import HeliosLogo from "../images/helios_logo.inline.svg";
import BaseButton from "./BaseButton";
import tw from "twin.macro";

const HeaderButton = tw(BaseButton)`px-8`;

const NavLink = tw.a`
mx-4 my-auto 
text-lg 
hidden
md:block
border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 
transition duration-500
`;

// Tailwind usage with direct classes.
const Header = (): React.ReactElement => {
  return (
    <nav
      id="nav"
      className="absolute inset-x-0 top-0 flex flex-row justify-between content-center z-10 text-white bg-transparent"
    >
      <div className="p-4">
        <a href="#" className="transition duration-500 hover:text-indigo-500">
          <HeliosLogo />
        </a>
      </div>

      <div className="p-4 md:flex flex-row justify-between font-sans uppercase">
        <NavLink href="#">The Conference</NavLink>
        <NavLink href="#partners">Partners</NavLink>
        <NavLink href="#">Press</NavLink>
        <HeaderButton>Attend</HeaderButton>
      </div>
    </nav>
  );
};

export default Header;
