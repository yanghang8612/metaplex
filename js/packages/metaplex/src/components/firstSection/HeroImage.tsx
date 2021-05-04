import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

import { BgImage } from "gbimage-bridge";

const EllipsesHeaderSVG =
  "data:image/svg+xml,%3Csvg%20viewBox='0%200%20218%20477'%20fill='none'%20xmlns='http://www.w3.org/2000/svg'%3E%3Ccircle%20cx='300'%20cy='177'%20r='120.5'%20stroke='%23DC3A34'%20stroke-opacity='0.5'/%3E%3Ccircle%20cx='300'%20cy='177'%20r='180.5'%20stroke='%23DC3A34'%20stroke-opacity='0.5'/%3E%3Ccircle%20cx='300'%20cy='177'%20r='239.5'%20stroke='%23DC3A34'%20stroke-opacity='0.5'/%3E%3Ccircle%20cx='300'%20cy='177'%20r='299.5'%20stroke='%23DC3A34'%20stroke-opacity='0.5'/%3E%3C/svg%3E";

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
      {
        index: file(relativePath: { eq: "first_image.jpg" }) {
          childImageSharp {
            gatsbyImageData(quality: 90, placeholder: NONE, layout: FULL_WIDTH)
          }
        }
      }
    `
  );

  // Set ImageData including the mobile background.
  const imageData = [
    { tracedSVG: EllipsesHeaderSVG, src: "", srcSet: "", aspectRatio: 0.5 },
    {
      ...index.childImageSharp.gatsbyImageData,
      media: `(min-width: 640px)`,
    },
  ];

  return (
    <BgImage
      Tag="div"
      className={className}
      // @ts-ignore
      image={imageData}
      style={{
        backgroundPosition: "",
        backgroundSize: "",
      }}
      backgroundColor={`#0c0e1b`}
    >
      {children}
    </BgImage>
  );
};

const StyledHeroImage = styled(HeroImage)`
  ${tw`
    min-h-screen w-full
    py-0 inset-0 
    flex flex-col justify-center 
  `}
  &:before,
  &:after {
    ${tw`bg-right-top bg-half sm:bg-right-bottom lg:bg-contain`}
  }
`;

export default StyledHeroImage;
