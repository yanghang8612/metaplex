import * as React from "react";

type PartnerCardProps = {
  partnerName: string;
  lastRow: boolean;
};

const PartnerCard = ({ partnerName, lastRow }: PartnerCardProps) => (
  <div
    className={`border-l border-gray-800 pt-5 ${lastRow ? `pb-28` : `pb-5`}`}
  >
    <img src={`/partners/${partnerName}.svg`} alt={partnerName} />
  </div>
);

export const PartnersPerCategory = ({ partners }: { partners: any }): any => {
  const partnerKeys = Object.keys(partners);

  return partnerKeys.map((category, index) => {
    const categoryTitle = `${category} Partners`;
    const lastRow = index === partnerKeys.length - 1;

    const partnersInCategory = partners[category]
      .split(",")
      .map((partnerName: any) => (
        <PartnerCard
          partnerName={partnerName.trim()}
          lastRow={lastRow}
          key={partnerName}
        />
      ));
    return (
      <div className="relative flex flex-row" key={category}>
        <h5 className="absolute left-2 top-1">{categoryTitle}</h5>
        {partnersInCategory}
        <div className="min-h-full border-r border-gray-800" />
      </div>
    );
  });
};

export const PartnerDisplay = ({ partners }: { partners: any }): any => (
  <>
    <PartnersPerCategory partners={partners} />
  </>
);

export default PartnerDisplay;
