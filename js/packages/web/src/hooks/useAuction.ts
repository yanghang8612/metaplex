import {
  TokenAccount,
  useConnection,
  useUserAccounts,
  useWallet,
} from '@oyster/common';
import { useEffect, useState } from 'react';
import { AuctionView, processAccountsIntoAuctionView } from '.';
import { useMeta } from '../contexts';

export const useAuction = (id: string) => {
  const { wallet } = useWallet();

  const [existingAuctionView, setAuctionView] = useState<AuctionView | null>(
    null,
  );

  const {
    auctions,
    auctionManagersByAuction,
    safetyDepositBoxesByVaultAndIndex,
    metadataByMint,
    bidderMetadataByAuctionAndBidder,
    bidderPotsByAuctionAndBidder,
    masterEditions,
    bidRedemptions,
    vaults,

    masterEditionsByMasterMint,
    metadataByMasterEdition,
  } = useMeta();

  useEffect(() => {
    const auction = auctions[id];
    if (auction && wallet) {
      const auctionView = processAccountsIntoAuctionView(
        wallet,
        auction,
        auctionManagersByAuction,
        safetyDepositBoxesByVaultAndIndex,
        metadataByMint,
        bidRedemptions,
        bidderMetadataByAuctionAndBidder,
        bidderPotsByAuctionAndBidder,
        masterEditions,
        vaults,
        masterEditionsByMasterMint,
        metadataByMasterEdition,
        undefined,
        existingAuctionView || undefined,
      );
      if (auctionView) setAuctionView(auctionView);
    }
  }, [
    auctions,
    wallet,
    auctionManagersByAuction,
    safetyDepositBoxesByVaultAndIndex,
    metadataByMint,
    bidderMetadataByAuctionAndBidder,
    bidderPotsByAuctionAndBidder,
    vaults,
    masterEditions,
    bidRedemptions,
    masterEditionsByMasterMint,
    metadataByMasterEdition,
  ]);
  return existingAuctionView;
};
