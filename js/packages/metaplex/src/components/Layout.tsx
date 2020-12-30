import * as React from "react";

import Header from "./header";
// import Footer from "./footer";
import SEO from "./SEO";

interface ILayoutProps {
  children?: React.ReactNode;
}

const Layout = ({ children }: ILayoutProps) => {
  return (
    <>
      <SEO title={`Helios`} />
      <Header />
      {children}
      {/*<Footer />*/}
    </>
  );
};

export default Layout;
