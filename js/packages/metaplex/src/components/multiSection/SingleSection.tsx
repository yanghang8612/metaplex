import * as React from "react";
import tw, { styled } from "twin.macro";

import SectionDisplay from "./SectionDisplay";

type SectionProps = {
  backgroundColor?: string;
};

const SectionWrapper = styled.div<SectionProps>`
  ${tw`
    min-h-halfScreen w-full
    py-5 md:py-20
    flex flex-col md:flex-row items-center
  `}
  h3 {
    ${tw`px-5 md:px-10`}
  }
  p {
    ${tw`px-5 md:px-10`}
  }
  background-color: ${(props) =>
    props.backgroundColor === "light" ? `#121212` : `transparent`};
`;

const ContainerText = styled.div`
  ${tw`
    w-full md:w-1/2
    text-white text-left
  `}
`;

const ContainerDisplay = tw.div`
  w-full md:w-1/2
  flex flex-row items-center justify-center
  text-white
`;

const RoundedSectionDisplay = styled(SectionDisplay)`
  ${tw`rounded-lg`}
  width: 640px;
  height: 440px;
  max-width: 90%;
`;

const SectionBgGradient = styled(RoundedSectionDisplay)`
  ${tw`flex items-end`}
  background: linear-gradient(135deg, #eca572 0%, #155fce 100%);
  backdrop-filter: blur(600px);
`;

type SingleSectionProps = {
  name: string;
  title?: string;
  position?: string;
  display?: string;
  background?: string;
  imageWrapper?: string;
  rawMarkdownBody?: string;
};

/**
 * Single Multi Section with position switch.
 *
 * @constructor
 */
const SingleSection = ({
  name,
  title,
  position,
  display,
  background,
  imageWrapper,
  rawMarkdownBody,
}: SingleSectionProps): React.ReactElement => {
  return (
    <SectionWrapper backgroundColor={background}>
      {position === "left" ? (
        <>
          <ContainerText>
            <h3>{title}</h3>
            <p>{rawMarkdownBody}</p>
          </ContainerText>
          <ContainerDisplay>
            {imageWrapper === "gradient" ? (
              <SectionBgGradient name={name} display={display} />
            ) : (
              <RoundedSectionDisplay name={name} display={display} />
            )}
          </ContainerDisplay>
        </>
      ) : (
        <>
          <ContainerDisplay>
            {imageWrapper === "gradient" ? (
              <SectionBgGradient name={name} display={display} />
            ) : (
              <RoundedSectionDisplay name={name} display={display} />
            )}
          </ContainerDisplay>
          <ContainerText>
            <h3>{title}</h3>
            <p>{rawMarkdownBody}</p>
          </ContainerText>
        </>
      )}
    </SectionWrapper>
  );
};

export default SingleSection;
