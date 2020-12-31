import * as React from "react";

import Header from "./Header";
// import Footer from "./footer";
import SEO from "./SEO";

interface ILayoutProps {
  children?: React.ReactNode;
}

const Layout = ({ children }: ILayoutProps): React.ReactElement => {
  return (
    <>
      <SEO title={`Helios`} />
      <Header />
      <main>{children}</main>
      {/*<Footer />*/}
    </>
  );
};

export default Layout;
