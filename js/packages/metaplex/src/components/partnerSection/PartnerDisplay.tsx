import * as React from "react";
import tw, { styled } from "twin.macro";

type PartnerCardProps = {
  partnerName: string;
};

const PartnerCard = ({ partnerName }: PartnerCardProps) => (
  <div>{partnerName}</div>
);

export const partnersByCategory = (partners: any) =>
  Object.keys(partners).map((category) => {
    const partnersInCategory = partners[category]
      .split(",")
      .map((partnerName: any) => (
        <PartnerCard partnerName={partnerName} key={partnerName} />
      ));
    const categoryTitle = `${category} Partners`;
    return (
      <div key={category}>
        <h5>{categoryTitle}</h5>
        {partnersInCategory}
      </div>
    );
  });

const PartnerDisplay = ({ partners }: { partners: any }) => {
  return <div>{partnersByCategory(partners)}</div>;
};

const StyledPartnerDisplay = styled(PartnerDisplay)`
  ${tw`bg-white border-white w-full py-64`}
`;

export default StyledPartnerDisplay;
