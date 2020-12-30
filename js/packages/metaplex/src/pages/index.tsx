import * as React from "react";
import tw, { styled } from "twin.macro";

import Layout from "../components/Layout";

// Styled version with macro, usage like in styled components.
const Container = styled.div`
  ${tw`relative sm:max-w-xl sm:mx-auto text-white text-xl text-center`}
  h1 {
    ${tw`text-4xl font-bold`}
  }
`;

// Shorthand Version.
const Button = tw.button`
  bg-green-500 hover:bg-green-800 text-white p-3 rounded mt-5
`;

const IndexPage = (): React.ReactElement => (
  <Layout>
    <Container>
      <h1>Hi from the Initial Helios Test!</h1>
      <Button>Activate</Button>
    </Container>
  </Layout>
);

export default IndexPage;
