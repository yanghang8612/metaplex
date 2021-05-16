import {
  TokenAccount,
  useConnection,
  useUserAccounts,
  useWallet,
} from '@oyster/common';
import { useEffect, useState } from 'react';
import {
  AuctionView,
  processAccountsIntoAuctionView,
  useCachedRedemptionKeysByWallet,
} from '.';
import { useMeta } from '../contexts';

export const useAuction = (id: string) => {
  const { wallet } = useWallet();
  const cachedRedemptionKeys = useCachedRedemptionKeysByWallet();

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
        bidderMetadataByAuctionAndBidder,
        bidderPotsByAuctionAndBidder,
        masterEditions,
        vaults,
        masterEditionsByMasterMint,
        metadataByMasterEdition,
        cachedRedemptionKeys,
        undefined,
        existingAuctionView || undefined,
      );
      if (auctionView) setAuctionView(auctionView);
    }
  }, [
    auctions,
    wallet?.publicKey,
    auctionManagersByAuction,
    safetyDepositBoxesByVaultAndIndex,
    metadataByMint,
    bidderMetadataByAuctionAndBidder,
    bidderPotsByAuctionAndBidder,
    vaults,
    masterEditions,
    masterEditionsByMasterMint,
    metadataByMasterEdition,
    cachedRedemptionKeys,
  ]);
  return existingAuctionView;
};
