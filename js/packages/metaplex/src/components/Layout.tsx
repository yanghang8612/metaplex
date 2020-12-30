import * as React from "react";

import Header from "./header";
// import Footer from "./footer";
import SEO from "./SEO";
import tw, { styled } from "twin.macro";

interface ILayoutProps {
  children?: React.ReactNode;
}

const Main = styled.main`
  ${tw`min-h-screen py-0 inset-0 bg-black flex flex-col justify-center`}
`;

const Layout = ({ children }: ILayoutProps) => {
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
