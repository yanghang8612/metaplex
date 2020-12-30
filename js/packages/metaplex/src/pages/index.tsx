import * as React from "react";
import tw from "twin.macro";

import Layout from "../components/Layout";

// Shorthand Versions.
const Container = tw.div`
  relative sm:max-w-xl sm:mx-auto text-blue-400 text-lg text-center
`;

const Button = tw.button`
  bg-blue-500 hover:bg-blue-800 text-white p-3 rounded mt-5
`;

const IndexPage = () => (
  <Layout>
    <Container>
      <h1>Hi from the Initial Helios Test!</h1>
      <Button>Activate</Button>
    </Container>
  </Layout>
);

export default IndexPage;
