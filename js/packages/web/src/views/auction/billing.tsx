import React, { useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import { Row, Col, Layout, Spin, Button } from 'antd';
import {
  useArt,
  useAuction,
  AuctionView,
  useBidsForAuction,
} from '../../hooks';
import { ArtContent } from '../../components/ArtContent';
import {
  useConnection,
  useUserAccounts,
  contexts,
  BidderMetadata,
  ParsedAccount,
  cache,
  BidderPot,
  fromLamports,
  useMint,
} from '@oyster/common';
import { useMeta } from '../../contexts';
import {
  getBidderKeys,
  NonWinningConstraint,
  WinningConstraint,
} from '../../models/metaplex';
import './billing.less';
import { WalletAdapter } from '@solana/wallet-base';
import { Connection, PublicKey } from '@solana/web3.js';
import { settle } from '../../actions/settle';
import { MintInfo } from '@solana/spl-token';
const { useWallet } = contexts.Wallet;
const { Content } = Layout;

export const BillingView = () => {
  const { id } = useParams<{ id: string }>();
  const auctionView: AuctionView | null = useAuction(id);
  const connection = useConnection();
  const { wallet } = useWallet();
  const mint = useMint(auctionView?.auction.info.tokenMint);

  return auctionView && wallet && connection && mint ? (
    <InnerBillingView
      auctionView={auctionView}
      connection={connection}
      wallet={wallet}
      mint={mint}
    />
  ) : (
    <Spin />
  );
};

function getLosingOpenEditionPrice(
  el: ParsedAccount<BidderMetadata>,
  auctionView: AuctionView,
) {
  const nonWinnerConstraint =
    auctionView.auctionManager.info.settings.openEditionNonWinningConstraint;

  if (nonWinnerConstraint === NonWinningConstraint.GivenForFixedPrice)
    return (
      auctionView.auctionManager.info.settings.openEditionFixedPrice?.toNumber() ||
      0
    );
  else if (nonWinnerConstraint === NonWinningConstraint.GivenForBidPrice)
    return el.info.lastBid.toNumber() || 0;
  else return 0;
}

export const InnerBillingView = ({
  auctionView,
  wallet,
  connection,
  mint,
}: {
  auctionView: AuctionView;
  wallet: WalletAdapter;
  connection: Connection;
  mint: MintInfo;
}) => {
  const {
    bidRedemptions,
    bidderMetadataByAuctionAndBidder,
    bidderPotsByAuctionAndBidder,
  } = useMeta();
  const art = useArt(auctionView.thumbnail.metadata.pubkey);
  const [escrowBalance, setEscrowBalance] = useState<number | undefined>();
  const [escrowBalanceRefreshCounter, setEscrowBalanceRefreshCounter] =
    useState(0);
  const auctionKey = auctionView.auction.pubkey.toBase58();

  useEffect(() => {
    connection
      .getTokenAccountBalance(auctionView.auctionManager.info.acceptPayment)
      .then(resp => {
        if (resp.value.uiAmount !== undefined && resp.value.uiAmount !== null)
          setEscrowBalance(resp.value.uiAmount);
      });
  }, [escrowBalanceRefreshCounter]);

  const { accountByMint } = useUserAccounts();
  const [openEditionBidRedemptionKeys, setOpenEditionBidRedemptionKeys] =
    useState<Record<string, PublicKey>>({});

  const bids = useBidsForAuction(auctionView.auction.pubkey);

  const myPayingAccount = accountByMint.get(
    auctionView.auction.info.tokenMint.toBase58(),
  );

  const uncancelledBids = bids.filter(b => !b.info.cancelled);

  const winners = auctionView.auction.info.bidState.bids;

  const winnersByBidderKey = winners.reduce(
    (mapper: Record<string, ParsedAccount<BidderPot>>, w) => {
      const nextEl = cache.get(w.key) as ParsedAccount<BidderPot>;
      if (nextEl) mapper[nextEl.info.bidderAct.toBase58()] = nextEl;
      return mapper;
    },
    {},
  );
  let hasOpenEdition =
    auctionView.auctionManager.info.settings.openEditionConfig !== undefined &&
    auctionView.auctionManager.info.settings.openEditionConfig !== null;
  let openEditionEligible = hasOpenEdition ? uncancelledBids : [];

  useMemo(async () => {
    const newKeys: Record<string, PublicKey> = {};

    for (let i = 0; i < openEditionEligible.length; i++) {
      const o = openEditionEligible[i];
      if (!openEditionBidRedemptionKeys[o.pubkey.toBase58()]) {
        newKeys[o.pubkey.toBase58()] = (
          await getBidderKeys(auctionView.auction.pubkey, o.info.bidderPubkey)
        ).bidRedemption;
      }
    }

    setOpenEditionBidRedemptionKeys({
      ...openEditionBidRedemptionKeys,
      ...newKeys,
    });
  }, [openEditionEligible.length]);

  if (
    auctionView.auctionManager.info.settings.openEditionWinnerConstraint ==
    WinningConstraint.NoOpenEdition
  )
    // Filter winners out of the open edition eligible
    openEditionEligible = openEditionEligible.filter(
      // winners are stored by pot key, not bidder key, so we translate
      b => !winnersByBidderKey[b.info.bidderPubkey.toBase58()],
    );

  const nonWinnerConstraint =
    auctionView.auctionManager.info.settings.openEditionNonWinningConstraint;

  const openEditionEligibleRedeemable: ParsedAccount<BidderMetadata>[] = [];
  const openEditionEligibleUnredeemable: ParsedAccount<BidderMetadata>[] = [];
  const openEditionEligibleRedeemed: ParsedAccount<BidderMetadata>[] = [];

  openEditionEligible.forEach(o => {
    const isWinner = winnersByBidderKey[o.info.bidderPubkey.toBase58()];
    // Winners automatically pay nothing for open editions, and are getting claimed anyway right now
    // so no need to add them to list
    if (isWinner) {
      return;
    }

    if (
      nonWinnerConstraint === NonWinningConstraint.GivenForFixedPrice ||
      nonWinnerConstraint === NonWinningConstraint.GivenForBidPrice
    ) {
      const key = openEditionBidRedemptionKeys[o.pubkey.toBase58()];
      if (key) {
        const redemption = bidRedemptions[key.toBase58()];
        if (!redemption || !redemption.info.openEditionRedeemed)
          openEditionEligibleUnredeemable.push(o);
        else if (redemption && redemption.info.openEditionRedeemed)
          openEditionEligibleRedeemed.push(o);
      } else openEditionEligibleUnredeemable.push(o);
    }
  });

  const openEditionUnredeemedTotal = openEditionEligibleUnredeemable.reduce(
    (acc, el) => (acc += getLosingOpenEditionPrice(el, auctionView)),
    0,
  );

  // Winners always get it for free so pay zero for them - figure out among all
  // eligible open edition winners what is the total possible for display.
  const openEditionPossibleTotal = openEditionEligible.reduce((acc, el) => {
    const isWinner = winnersByBidderKey[el.info.bidderPubkey.toBase58()];
    let price = 0;
    if (!isWinner) price = getLosingOpenEditionPrice(el, auctionView);

    return (acc += price);
  }, 0);

  const totalWinnerPayments = winners.reduce(
    (acc, w) => (acc += w.amount.toNumber()),
    0,
  );

  const winnersThatCanBeEmptied = Object.values(winnersByBidderKey).filter(
    p => !p.info.emptied,
  );

  const emptiedBids = bids.filter(
    b =>
      bidderPotsByAuctionAndBidder[
        `${auctionKey}-${b.info.bidderPubkey.toBase58()}`
      ]?.info.emptied,
  );

  const totalMovedToEscrowViaClaims = emptiedBids.reduce(
    (acc, el) => (acc += el.info.cancelled ? 0 : el.info.lastBid.toNumber()),
    0,
  );

  const totalMovedToEscrowAsOpenEditionPayments =
    openEditionEligibleRedeemed.reduce(
      (acc, el) => (acc += getLosingOpenEditionPrice(el, auctionView)),
      0,
    );

  const bidsToClaim: {
    metadata: ParsedAccount<BidderMetadata>;
    pot: ParsedAccount<BidderPot>;
  }[] = [
    ...winnersThatCanBeEmptied.map(pot => ({
      metadata:
        bidderMetadataByAuctionAndBidder[
          `${auctionKey}-${pot.info.bidderAct.toBase58()}`
        ],
      pot,
    })),
    ...openEditionEligibleRedeemable.map(metadata => ({
      metadata,
      pot: bidderPotsByAuctionAndBidder[
        `${auctionKey}-${metadata.info.bidderPubkey.toBase58()}`
      ],
    })),
  ];

  return (
    <Content>
      <Col>
        <Row
          style={{ margin: '0 30px', textAlign: 'left', fontSize: '1.4rem' }}
        >
          <Col span={12}>
            <ArtContent
              category={art.category}
              content={art.image}
              className="artwork-image"
            />
          </Col>
          <Col span={12}>
            <div style={{ fontWeight: 700 }}>{art.title}</div>
            <br />
            <div className="info-header">TOTAL AUCTION VALUE</div>
            <div className="escrow">
              ◎
              {fromLamports(
                totalWinnerPayments + openEditionPossibleTotal,
                mint,
              )}
            </div>
            <br />
            <div className="info-header">TOTAL AUCTION REDEEMED VALUE</div>
            <div className="escrow">
              ◎
              {fromLamports(
                totalWinnerPayments +
                  openEditionPossibleTotal -
                  openEditionUnredeemedTotal,
                mint,
              )}
            </div>
            <br />
            <div className="info-header">TOTAL COLLECTED BY ARTIST</div>
            <div className="escrow">
              {escrowBalance !== undefined ? (
                `◎${
                  fromLamports(
                    totalMovedToEscrowViaClaims +
                      totalMovedToEscrowAsOpenEditionPayments,
                    mint,
                  ) - escrowBalance
                }`
              ) : (
                <Spin />
              )}
            </div>
            <br />
            <div className="info-header">TOTAL UNSETTLED</div>
            <div className="escrow">
              ◎
              {fromLamports(
                bidsToClaim.reduce(
                  (acc, el) => (acc += el.metadata.info.lastBid.toNumber()),
                  0,
                ),
                mint,
              )}
            </div>
            <br />
            <div className="info-header">TOTAL IN ESCROW</div>
            <div className="escrow">
              {escrowBalance !== undefined ? `◎${escrowBalance}` : <Spin />}
            </div>
            <br />
            {hasOpenEdition && (
              <>
                <div className="info-header">
                  TOTAL UNREDEEMED PARTICIPATION FEES OUTSTANDING
                </div>
                <div className="outstanding-open-editions">
                  ◎{fromLamports(openEditionUnredeemedTotal, mint)}
                </div>
                <br />
              </>
            )}
            <br />
            <Button
              type="primary"
              size="large"
              className="action-btn"
              disabled={!myPayingAccount}
              onClick={async () => {
                if (myPayingAccount)
                  await settle(
                    connection,
                    wallet,
                    auctionView,
                    myPayingAccount,
                    bidsToClaim.map(b => b.pot),
                  );
                setEscrowBalanceRefreshCounter(ctr => ctr + 1);
              }}
            >
              SETTLE OUTSTANDING
            </Button>
          </Col>
        </Row>
      </Col>
    </Content>
  );
};
