import * as React from "react";
import tw, { styled } from "twin.macro";

// Styled version (like in styled components).
const Main = styled.main`
  ${tw`min-h-screen py-0 inset-0 bg-black flex flex-col justify-center`}
`;

const Container = tw.div`
  relative sm:max-w-xl sm:mx-auto
`;

// Shorthand Version.
const Button = tw.button`
  bg-blue-500 hover:bg-blue-800 text-white p-2 rounded
`;

const IndexPage = () => (
  <Main>
    <Container>
      <h1>Hi people</h1>
      <Button>Activate</Button>
    </Container>
  </Main>
);

export default IndexPage;
