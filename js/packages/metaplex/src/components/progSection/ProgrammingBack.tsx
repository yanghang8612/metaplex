import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import BackgroundImage from "gatsby-background-image";

type Props = {
  children?: React.ReactNode;
  className?: string;
};

const ProgrammingSectionBackground = ({
  className,
  children,
  ...props
}: Props): React.ReactElement => {
  const { prog } = useStaticQuery(
    graphql`
      {
        prog: file(relativePath: { eq: "prog_image.png" }) {
          childImageSharp {
            gatsbyImageData(quality: 90, placeholder: NONE, layout: FULL_WIDTH)
          }
        }
      }
    `
  );

  // Set ImageData.
  const imageData = prog.childImageSharp.gatsbyImageData;

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

const StyledProgrammingSectionBackground = styled(ProgrammingSectionBackground)`
  ${tw`w-full py-0 flex flex-col justify-center`}//&:before,
  //&:after {
  //  filter: brightness(70%);
  //}
`;

export default StyledProgrammingSectionBackground;
