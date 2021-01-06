import { Global } from "@emotion/react";
import * as React from "react";
import { GlobalStyles } from "twin.macro";
import globalStyles from "../utils/globalStyles";

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
      <GlobalStyles />
      <Global styles={globalStyles} />
      <main>
        {children}
        <Header />
      </main>
      {/*<Footer />*/}
    </>
  );
};

export default Layout;
