import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import BackgroundImage from "gatsby-background-image";

type Props = {
  children?: React.ReactNode;
  className?: string;
};

/**
 * Hero Image - To be replaced by video.
 *
 * @param className
 * @param children
 * @constructor
 */
const HeroImage = ({ className, children }: Props): React.ReactElement => {
  const { index } = useStaticQuery(
    graphql`
      query {
        index: file(relativePath: { eq: "first_image.jpg" }) {
          childImageSharp {
            fluid(quality: 90, maxWidth: 1920) {
              ...GatsbyImageSharpFluid_withWebp_noBase64
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
      backgroundColor={`#0c0e1b`}
    >
      {children}
    </BackgroundImage>
  );
};

const StyledHeroImage = styled(HeroImage)`
  ${tw`min-h-screen w-full py-0 inset-0 flex flex-col justify-center bg-right bg-contain`}
`;

export default StyledHeroImage;
