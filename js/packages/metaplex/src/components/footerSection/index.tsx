import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import BaseButton from "../BaseButton";

const Container = styled.div`
  ${tw`
    flex flex-col justify-center items-center
    text-white py-10 md:py-44 md:pb-52
  `}
`;

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
            button
          }
          rawMarkdownBody
        }
      }
    `
  );
  return (
    <>
      <Container>
        <h3>{footer.frontmatter.title}</h3>
        <p>{footer.rawMarkdownBody}</p>
        <BaseButton>{footer.frontmatter.button}</BaseButton>
      </Container>
    </>
  );
};

export default FooterSection;
