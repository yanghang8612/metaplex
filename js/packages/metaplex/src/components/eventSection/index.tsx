import * as React from "react";
import { graphql, useStaticQuery } from "gatsby";
import tw, { styled } from "twin.macro";

const EventSectionWrapper = styled.div`
  ${tw`w-full p-4 sm:py-16 block md:flex flex-row`}
  background-color: #0c0e1b;
`;

const ContainerLeft = styled.div`
  ${tw`w-full md:w-2/3
    text-white text-xl text-left
    py-3 sm:py-16 
    px-2 sm:px-8`}
  h2 {
    ${tw`mt-2 sm:mt-5`}
  }
`;

const ContainerRight = tw.div`
  w-full md:w-1/5
  justify-end
  my-auto
  mx-auto
  text-white
`;

/**
 * Event section.
 *
 * @constructor
 */
const EventSection = (): React.ReactElement => {
  const { event } = useStaticQuery(
    graphql`
      query {
        event: markdownRemark(frontmatter: { slug: { eq: "event" } }) {
          frontmatter {
            title
            summary
          }
          rawMarkdownBody
        }
      }
    `
  );
  return (
    <EventSectionWrapper id="event">
      <ContainerLeft>
        <h5>{event.frontmatter.summary}</h5>
        <h2>{event.frontmatter.title}</h2>
      </ContainerLeft>
      <ContainerRight>
        <p>{event.rawMarkdownBody}</p>
      </ContainerRight>
    </EventSectionWrapper>
  );
};

export default EventSection;
