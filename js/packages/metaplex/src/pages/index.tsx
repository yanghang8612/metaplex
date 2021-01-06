import * as React from "react";

import Layout from "../components/Layout";
import FirstSection from "../components/firstSection";
import PartnerSection from "../components/partnerSection";
import ProgrammingSection from "../components/progSection";
import EventSection from "../components/eventSection";

const IndexPage = (): React.ReactElement => (
  <Layout>
    <FirstSection />
    <EventSection />
    <PartnerSection />
    <ProgrammingSection />
  </Layout>
);

export default IndexPage;
