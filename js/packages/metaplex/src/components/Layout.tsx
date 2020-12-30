import * as React from "react";

import Header from "./header";
// import Footer from "./footer";
import SEO from "./SEO";
import tw, { styled } from "twin.macro";
import StyledHeroImage from "./header/HeroImage";

interface ILayoutProps {
  children?: React.ReactNode;
}

// Styled version with macro, usage like in styled components.
const Main = styled(StyledHeroImage)`
  ${tw`min-h-screen py-0 inset-0 bg-black flex flex-col justify-center`}
`;

const Layout = ({ children }: ILayoutProps): React.ReactElement => {
  return (
    <>
      <SEO title={`Helios`} />
      <Main>
        <Header />
        {children}
      </Main>
      {/*<Footer />*/}
    </>
  );
};

export default Layout;
