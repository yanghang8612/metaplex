import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import StyledFooterImage from "./FooterImage";
import Divider from "../Divider";
import Footer from "./Footer";
import FooterNav from "./FooterNav";

const OuterContainer = styled.div`
  ${tw`flex text-white text-xl`}
`;

const Container = styled.div`
  ${tw`
    sm:max-w-xl md:max-w-3xl
    text-white text-xl text-right 
    py-24 
    px-2 sm:px-10`}
`;

const HeroDivider = styled(Divider)`
  ${tw`mt-7 mb-5`}
  max-width: 60%;
  margin-left: -0.5rem;
`;

const LeftWrapper = tw.div`flex-grow`;

/**
 * Footer section.
 *
 * @constructor
 */
const FooterSection = (): React.ReactElement => {
  const { footer } = useStaticQuery(
    graphql`
      query {
        footer: markdownRemark(frontmatter: { slug: { eq: "footer" } }) {
          frontmatter {
            title
          }
          rawMarkdownBody
        }
      }
    `
  );
  return (
    <StyledFooterImage>
      <OuterContainer>
        <LeftWrapper />
        <Container>
          <h2>{footer.frontmatter.title}</h2>
          <p>{footer.rawMarkdownBody}</p>
        </Container>
      </OuterContainer>
      <FooterNav />
      <Footer />
    </StyledFooterImage>
  );
};

export default FooterSection;
