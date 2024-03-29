import { MetadataCategory } from '@oyster/common';

export interface Auction {
  name: string;
  auctionerName: string;
  auctionerLink: string;
  highestBid: number;
  solAmt: number;
  link: string;
  image: string;

  endingTS: number;
}

export interface Artist {
  address?: string;
  name: string;
  link: string;
  image: string;
  itemsAvailable?: number;
  itemsSold?: number;
  about?: string;
}

export enum ArtType {
  Master,
  Print,
  NFT,
}
export interface Art {
  image: string;
  category: MetadataCategory;
  link: string;
  title: string;
  artist: string;
  priceSOL: number;
  priceUSD?: number;
  endingTS?: number;
  royalties?: number;
  about?: string;
  type: ArtType;
  edition?: number;
  supply?: number;
  maxSupply?: number;
}

export interface Presale {
  endingTS: number;
  targetPricePerShare?: number;
  pricePerShare?: number;
  marketCap?: number;
}
