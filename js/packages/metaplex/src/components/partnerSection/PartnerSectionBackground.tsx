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
      query {
        second: file(relativePath: { eq: "second_image.jpg" }) {
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
  const imageData = second.childImageSharp.fluid;

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
`;

export default StyledPartnerSectionBackground;
