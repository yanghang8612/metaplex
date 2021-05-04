import { Global } from "@emotion/react";
import * as React from "react";
import { GlobalStyles } from "twin.macro";
import globalStyles from "../utils/globalStyles";
import FooterSection from "./footerSection";

import SEO from "./SEO";

interface ILayoutProps {
  children?: React.ReactNode;
}

const Layout = ({ children }: ILayoutProps): React.ReactElement => {
  return (
    <>
      <SEO title={`Metaplex`} />
      <GlobalStyles />
      <Global styles={globalStyles} />
      <main>
        {children}
        {/*<FooterSection />*/}
      </main>
    </>
  );
};

export default Layout;
