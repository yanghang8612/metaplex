import { Global } from "@emotion/react";
import * as React from "react";
import { GlobalStyles } from "twin.macro";
import globalStyles from "../utils/globalStyles";
import FooterSection from "./footerSection";

import Header from "./Header";
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
        <FooterSection />
      </main>
    </>
  );
};

export default Layout;
