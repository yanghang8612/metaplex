import * as React from "react";
import tw, { styled } from "twin.macro";

import Layout from "../components/Layout";

// Styled version (like in styled components).
const Main = styled.main`
  ${tw`min-h-screen py-0 inset-0 bg-black flex flex-col justify-center`}
`;

// Shorthand Version.
const Container = tw.div`
  relative sm:max-w-xl sm:mx-auto text-blue-400 text-lg text-center
`;

const Button = tw.button`
  bg-blue-500 hover:bg-blue-800 text-white p-3 rounded mt-5
`;

const IndexPage = () => (
  <Layout>
    <Main>
      <Container>
        <h1>Hi from the Initial Helios Test!</h1>
        <Button>Activate</Button>
      </Container>
    </Main>
  </Layout>
);

export default IndexPage;
