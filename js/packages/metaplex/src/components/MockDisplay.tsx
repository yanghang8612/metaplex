import tw, { styled } from "twin.macro";

const MockDisplay = styled.div`
  ${tw`
    bg-gradient-to-l from-btnPrimaryStart to-btnPrimaryStop
    text-black
    grid place-content-center
  `}
  width: 380px;
  min-height: 300px;
`;

export default MockDisplay;
