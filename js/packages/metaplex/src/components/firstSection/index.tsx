import * as React from 'react';
import { graphql, useStaticQuery } from 'gatsby';
import tw, { styled } from 'twin.macro';

import MetaplexSVG from '../../images/metaplex.inline.svg';
import BaseButton from '../BaseButton';
import MockDisplay from '../MockDisplay';

const SectionWrapper = styled.div`
  ${tw`
    min-h-halfScreen w-full
    py-0 inset-0 
    flex flex-row justify-center 
  `}
`;

const ContainerLeft = styled.div`
  ${tw`w-full md:w-1/2
    text-white text-xl text-left
    py-3 sm:py-16 
    px-2 sm:px-8`}
`;

const ContainerRight = tw.div`
  relative
  w-full md:w-1/2
  text-white
`;

const MetaplexWrapper = styled.div`
  ${tw`
    w-9/12
    mb-20
    `}
`;

const BlueButton = tw(BaseButton)`
  bg-gradient-to-r from-btnBlueStart to-btnBlueStop
  text-sm
  px-8 py-3 
  mt-5
  mr-2
  mb-2
`;

const BottomMock = styled(MockDisplay)`
  position: absolute;
  top: 50px;
  left: 120px;
`;

const TopMock = styled(MockDisplay)`
  position: absolute;
  top: 110px;
  left: 200px;
`;

/**
 * Top section (Hero section).
 *
 * @constructor
 */
const FirstSection = (): React.ReactElement => {
  const { hero } = useStaticQuery(
    graphql`
      query {
        hero: markdownRemark(frontmatter: { slug: { eq: "hero" } }) {
          frontmatter {
            title
          }
          rawMarkdownBody
        }
      }
    `,
  );
  return (
    <SectionWrapper>
      <ContainerLeft>
        <MetaplexWrapper>
          <MetaplexSVG />
        </MetaplexWrapper>
        <h3>{hero.frontmatter.title}</h3>
        <p>{hero.rawMarkdownBody}</p>
        <BlueButton>Request an invite</BlueButton>
        <BaseButton>Request an invite</BaseButton>
      </ContainerLeft>
      <ContainerRight>
        <BottomMock />
        <TopMock>Product Mock</TopMock>
      </ContainerRight>
    </SectionWrapper>
  );
};

export default FirstSection;
