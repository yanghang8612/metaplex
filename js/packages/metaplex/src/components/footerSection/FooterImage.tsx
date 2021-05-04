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
const FooterImage = ({ className, children }: Props): React.ReactElement => {
  const { footer } = useStaticQuery(
    graphql`
      {
        footer: file(relativePath: { eq: "footer_image.jpg" }) {
          childImageSharp {
            gatsbyImageData(quality: 90, placeholder: NONE, layout: FULL_WIDTH)
          }
        }
      }
    `
  );

  // Set ImageData including the mobile background.
  const imageData = footer.childImageSharp.gatsbyImageData;

  return (
    <BackgroundImage
      Tag="div"
      className={className}
      fluid={imageData}
      style={{
        backgroundPosition: "",
        backgroundSize: "",
      }}
      backgroundColor={`#0a0c21`}
    >
      {children}
    </BackgroundImage>
  );
};

const StyledFooterImage = styled(FooterImage)`
  ${tw`
    w-full
    py-10 inset-0 
    flex flex-col justify-center 
  `}
  &:before,
  &:after {
    ${tw`bg-left bg-half lg:bg-contain`}
  }
`;

export default StyledFooterImage;
