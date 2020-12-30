import * as React from "react";
import HeliosLogo from "../../images/helios_logo.inline.svg";

// Tailwind usage with direct classes.
const Header = () => {
  return (
    <nav
      id="nav"
      className="fixed inset-x-0 top-0 flex flex-row justify-between z-10 text-white bg-transparent"
    >
      <div className="p-4">
        <div className="font-extrabold tracking-widest text-xl">
          <a href="#" className="transition duration-500 hover:text-indigo-500">
            <HeliosLogo />
          </a>
        </div>
      </div>

      <div className="p-4 hidden md:flex flex-row justify-between font-bold">
        <a
          id="hide-after-click"
          href="#"
          className="mx-4 text-lg  border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 transition duration-500"
        >
          The Conference
        </a>
        <a
          href="#"
          className="mx-4 text-lg border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 transition duration-500"
        >
          Partners
        </a>
        <a
          href="#"
          className="mx-4 text-lg border-b-2 border-transparent hover:border-b-2 hover:border-indigo-300 transition duration-500"
        >
          Press
        </a>
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
