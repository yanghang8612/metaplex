import * as React from 'react';
import { graphql, useStaticQuery } from 'gatsby';
import { GatsbyImage, getImage } from 'gatsby-plugin-image';
import MockDisplay from '../MockDisplay';

type Props = {
  display?: string;
  className?: string;
  name?: string;
};

const SectionDisplay = ({
  className,
  display,
  name,
}: Props): React.ReactElement => {
  const displayImage = useStaticQuery(
    graphql`
      {
        nftShop: file(relativePath: { eq: "nft_shop.png" }) {
          childImageSharp {
            gatsbyImageData(quality: 90, placeholder: NONE, layout: CONSTRAINED)
          }
        }
        nftExample: file(relativePath: { eq: "nft_example.png" }) {
          childImageSharp {
            gatsbyImageData(quality: 90, placeholder: NONE, layout: CONSTRAINED)
          }
        }
        collab: file(relativePath: { eq: "collab.png" }) {
          childImageSharp {
            gatsbyImageData(quality: 90, placeholder: NONE, layout: CONSTRAINED)
          }
        }
      }
    `,
  );

  return (
    <MockDisplay className={className}>
      {!display ? (
        <GatsbyImage image={getImage(displayImage[name])} alt={name} />
      ) : (
        <span>{display}</span>
      )}
    </MockDisplay>
  );
};

export default SectionDisplay;
