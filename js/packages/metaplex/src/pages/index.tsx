import * as React from "react";

import Layout from "../components/Layout";
import FirstSection from "../components/firstSection";
import CiteSection from "../components/citeSection";

const IndexPage = (): React.ReactElement => (
  <Layout>
    <FirstSection />
    <CiteSection />
    {/*<PartnerSection />*/}
    {/*<ProgrammingSection />*/}
  </Layout>
);

export default IndexPage;
