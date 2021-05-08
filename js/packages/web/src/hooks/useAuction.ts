import { TokenAccount, useConnection, useUserAccounts } from '@oyster/common';
import { useEffect, useState } from 'react';
import { AuctionView, processAccountsIntoAuctionView } from '.';
import { useMeta } from '../contexts';

export const useAuction = (id: string) => {
  const { userAccounts } = useUserAccounts();
  const accountByMint = userAccounts.reduce((prev, acc) => {
    prev.set(acc.info.mint.toBase58(), acc);
    return prev;
  }, new Map<string, TokenAccount>());
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
    if (auction) {
      const auctionView = processAccountsIntoAuctionView(
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
        accountByMint,
        undefined,
        existingAuctionView || undefined,
      );
      if (auctionView) setAuctionView(auctionView);
    }
  }, [
    auctions,
    auctionManagersByAuction,
    safetyDepositBoxesByVaultAndIndex,
    metadataByMint,
    bidderMetadataByAuctionAndBidder,
    bidderPotsByAuctionAndBidder,
    vaults,
    masterEditions,
    bidRedemptions,
    userAccounts,

    masterEditionsByMasterMint,
    metadataByMasterEdition,
  ]);
  return existingAuctionView;
};
