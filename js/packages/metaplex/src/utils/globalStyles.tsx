import tw, { css } from "twin.macro";

const globalStyles = css`
  html {
    scroll-behavior: smooth;
  }
  h1 {
    ${tw`text-6xl md:text-8xl font-header mb-6`}
  }
  h2 {
    ${tw`text-5xl md:text-6xl font-header mb-4`}
  }
  h3 {
    ${tw`text-2xl md:text-4xl font-header mb-3`}
  }
  h5 {
    ${tw`text-sm font-sans uppercase`}
    color: #DC3A34;
  }
  p {
    ${tw`font-serif mb-6`}
  }
`;

export default globalStyles;
