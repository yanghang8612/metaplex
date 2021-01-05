// Shorthand Version.
import tw from "twin.macro";

const BaseButton = tw.button`
  bg-transparent hover:bg-white
  font-sans
  text-white hover:text-black
  transition duration-500 
  p-2 
  border-2 border-white
  uppercase
`;

export default BaseButton;
