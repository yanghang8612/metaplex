import * as React from "react";

import Layout from "../components/Layout";
import FirstSection from "../components/firstSection";
import CiteSection from "../components/citeSection";
import WelcomeSection from "../components/welcomeSection";
import FooterSection from "../components/footerSection";

const IndexPage = (): React.ReactElement => (
  <Layout>
    <FirstSection />
    <CiteSection />
    <WelcomeSection />
    {/*<ProgrammingSection />*/}
    <FooterSection />
  </Layout>
);

export default IndexPage;
