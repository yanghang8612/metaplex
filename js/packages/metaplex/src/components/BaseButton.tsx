// Shorthand Version.
import tw from 'twin.macro';

const BaseButton = tw.button`
  bg-gradient-to-l from-btnPrimaryStart to-btnPrimaryStop
  hover:bg-gradient-to-tl
  rounded-lg
  font-sans
  text-sm text-white hover:text-black
  transition duration-500 
  px-8 py-3
`;

export default BaseButton;
