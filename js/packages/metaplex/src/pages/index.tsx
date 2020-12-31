import * as React from "react";

import Layout from "../components/Layout";
import FirstSection from "../components/firstSection";
import PartnerSection from "../components/partnerSection";

const IndexPage = (): React.ReactElement => (
  <Layout>
    <FirstSection />
    <PartnerSection />
  </Layout>
);

export default IndexPage;
