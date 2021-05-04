import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import BackgroundImage from "gatsby-background-image";

type Props = {
  children?: React.ReactNode;
  className?: string;
};

const PartnerSectionBackground = ({
  className,
  children,
  ...props
}: Props): React.ReactElement => {
  const { second } = useStaticQuery(
    graphql`
      {
        second: file(relativePath: { eq: "second_image.jpg" }) {
          childImageSharp {
            gatsbyImageData(quality: 90, placeholder: NONE, layout: FULL_WIDTH)
          }
        }
      }
    `
  );

  // Set ImageData.
  const imageData = second.childImageSharp.gatsbyImageData;

  return (
    <BackgroundImage
      Tag="div"
      className={className}
      fluid={imageData}
      backgroundColor={`#0c0e1b`}
      {...props}
    >
      {children}
    </BackgroundImage>
  );
};

const StyledPartnerSectionBackground = styled(PartnerSectionBackground)`
  ${tw`min-h-screen w-full py-0 inset-0 flex flex-col justify-center bg-left bg-contain`}
  &:before,
  &:after {
    filter: brightness(70%);
  }
`;

export default StyledPartnerSectionBackground;
