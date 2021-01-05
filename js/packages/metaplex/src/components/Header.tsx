import * as React from "react";
import HeliosLogo from "../images/helios_logo.inline.svg";
import BaseButton from "./BaseButton";
import tw from "twin.macro";

// TODO: split into (tw) components.

const HeaderButton = tw(BaseButton)`px-8`;

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

      <div className="p-4 hidden md:flex flex-row justify-between font-sans uppercase">
        <a
          id="hide-after-click"
          href="#"
          className="mx-4 my-auto text-lg border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 transition duration-500"
        >
          The Conference
        </a>
        <a
          href="#partners"
          className="mx-4 my-auto text-lg border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 transition duration-500"
        >
          Partners
        </a>
        <a
          href="#"
          className="mx-4 my-auto  border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 transition duration-500"
        >
          Press
        </a>
        <HeaderButton>Attend</HeaderButton>
      </div>

      <div id="nav-open" className="p-4 md:hidden">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="feather feather-menu"
        >
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
      </div>
    </nav>
  );
};

export default Header;
