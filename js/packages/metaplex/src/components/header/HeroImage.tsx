import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import { styled } from "twin.macro";

import BackgroundImage from "gatsby-background-image";

type Props = {
  children?: React.ReactNode;
  className?: string;
};

const HeroImage = ({ className, children }: Props) => {
  const { index } = useStaticQuery(
    graphql`
      query {
        index: file(relativePath: { eq: "dummy_hero.jpg" }) {
          childImageSharp {
            fluid(quality: 90, maxWidth: 1920) {
              ...GatsbyImageSharpFluid_withWebp
            }
          }
        }
      }
    `
  );

  // Set ImageData.
  const imageData = index.childImageSharp.fluid;

  return (
    <BackgroundImage
      Tag="div"
      className={className}
      fluid={imageData}
      backgroundColor={`#040e18`}
    >
      {children}
    </BackgroundImage>
  );
};

const StyledHeroImage = styled(HeroImage)`
  width: 100%;
  background-position: bottom center;
  background-repeat: repeat-y;
  background-size: cover;
`;

export default StyledHeroImage;
