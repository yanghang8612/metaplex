import React, { useEffect, useState } from 'react';
import { Row, Col, Button, InputNumber, Alert } from 'antd';

import './index.less';
import { getCountdown } from '../../utils/utils';
import {
  shortenAddress,
  TokenAccount,
  useConnection,
  useUserAccounts,
  contexts,
  BidderMetadata,
  ParsedAccount,
  Identicon,
  MetaplexModal,
  formatAmount,
} from '@oyster/common';
import {
  AuctionView,
  AuctionViewState,
  useBidsForAuction,
  useUserBalance,
} from '../../hooks';
import { sendPlaceBid } from '../../actions/sendPlaceBid';
import {
  sendRedeemBid,
  eligibleForOpenEditionGivenWinningIndex,
} from '../../actions/sendRedeemBid';
import { AmountLabel } from '../AmountLabel';
import { sendCancelBid } from '../../actions/cancelBid';
import BN from 'bn.js';

const { useWallet } = contexts.Wallet;

export const AuctionCard = ({ auctionView }: { auctionView: AuctionView }) => {
  const [days, setDays] = useState<number>(99);
  const [hours, setHours] = useState<number>(23);
  const [minutes, setMinutes] = useState<number>(59);
  const [seconds, setSeconds] = useState<number>(59);
  const [clock, setClock] = useState<number>(0);
  const connection = useConnection();
  const { wallet } = useWallet();
  const [value, setValue] = useState<number>();
  const [showMModal, setShowMModal] = useState<boolean>(false);
  const [lastBid, setLastBid] = useState<{ amount: BN } | undefined>(undefined);

  const { accountByMint } = useUserAccounts();

  const bids = useBidsForAuction(auctionView.auction.pubkey);

  const balance = useUserBalance(auctionView.auction.info.tokenMint);

  const myPayingAccount = balance.accounts[0];
  let winnerIndex = null;
  if (auctionView.myBidderPot?.pubkey)
    winnerIndex = auctionView.auction.info.bidState.getWinnerIndex(
      auctionView.myBidderPot?.pubkey,
    );

  const eligibleForOpenEdition = eligibleForOpenEditionGivenWinningIndex(
    winnerIndex,
    auctionView,
  );

  const eligibleForAnything = winnerIndex != null || eligibleForOpenEdition;

  useEffect(() => {
    connection.getSlot().then(setClock);
  }, [connection]);

  useEffect(() => {
    const interval = setInterval(() => {
      const { days, hours, minutes, seconds } = getCountdown(
        auctionView.auction.info.endAuctionAt?.toNumber() as number,
      );

      setDays(Math.min(days, 99));
      setHours(hours);
      setMinutes(minutes);
      setSeconds(seconds);
    }, 1000);
    return () => clearInterval(interval);
  }, [clock]);

  const isUpcoming = auctionView.state === AuctionViewState.Upcoming;
  const isStarted = auctionView.state === AuctionViewState.Live;

  return (
    <div className="presale-card-container">
      {isUpcoming && <AmountLabel title="STARTING BID" amount={40} />}
      {isStarted && <AmountLabel title="HIGHEST BID" amount={40} />}
      <br />
      {days === 0 && hours === 0 && minutes === 0 && seconds === 0 ? (
        <div className="info-header">AUCTION HAS ENDED</div>
      ) : (
        <>
          <div className="info-header">AUCTION ENDS IN</div>
          <Row style={{ width: 300 }}>
            {days > 0 && (
              <Col span={8}>
                <div className="cd-number">{days}</div>
                <div className="cd-label">days</div>
              </Col>
            )}
            <Col span={8}>
              <div className="cd-number">{hours}</div>
              <div className="cd-label">hours</div>
            </Col>
            <Col span={8}>
              <div className="cd-number">{minutes}</div>
              <div className="cd-label">minutes</div>
            </Col>
            {!days && (
              <Col span={8}>
                <div className="cd-number">{seconds}</div>
                <div className="cd-label">seconds</div>
              </Col>
            )}
          </Row>
        </>
      )}
      <br />
      <div
        className="info-content"
        style={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: '0.9rem' }}
      >
        Any bids placed in the last 15 minutes will extend the auction for
        another 15 minutes.
      </div>
      <br />

      <div className="info-line" />

      <InputNumber
        autoFocus
        className="input"
        value={value}
        style={{ width: '100%', backgroundColor: 'black', marginTop: 20 }}
        onChange={setValue}
        prefix="$"
        placeholder="Amount in USD"
      />

      <div
        className="info-content"
        style={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: '0.9rem' }}
      >
        Your Balance: ◎{formatAmount(balance.balance, 2)} ($
        {formatAmount(balance.balanceInUSD, 2)})
      </div>

      {auctionView.state === AuctionViewState.Ended ? (
        <Button
          type="primary"
          size="large"
          className="action-btn"
          disabled={!auctionView.myBidderMetadata}
          onClick={() => {
            console.log('Auctionview', auctionView);
            if (eligibleForAnything)
              sendRedeemBid(connection, wallet, auctionView, accountByMint);
            else sendCancelBid(connection, wallet, auctionView, accountByMint);
          }}
          style={{ marginTop: 20 }}
        >
          {eligibleForAnything ? 'REDEEM BID' : 'REFUND BID'}
        </Button>
      ) : (
        <Button
          type="primary"
          size="large"
          className="action-btn"
          disabled={!myPayingAccount || value === undefined}
          onClick={() => {
            console.log('Auctionview', auctionView);
            if (myPayingAccount && value) {
              sendPlaceBid(
                connection,
                wallet,
                myPayingAccount.pubkey,
                auctionView,
                value,
              ).then(bid => {
                setShowMModal(true);
                setLastBid(bid);
              });
            }
          }}
          style={{ marginTop: 20 }}
        >
          PLACE BID
        </Button>
      )}
      <AuctionBids bids={bids} />
      <MetaplexModal visible={showMModal} onCancel={() => setShowMModal(false)}>
        <h2>Congratulations!</h2>
        <p>Your bid has been placed</p>
        <br />
        {lastBid && <AmountLabel amount={lastBid.amount.toNumber()} />}
        <br />
        <Button className="metaplex-button" onClick={_ => setShowMModal(false)}>
          <span>Continue</span>
          <span>&gt;</span>
        </Button>
      </MetaplexModal>
    </div>
  );
};

export const AuctionBids = ({
  bids,
}: {
  bids: ParsedAccount<BidderMetadata>[];
}) => {
  return (
    <Col style={{ width: '100%' }}>
      {bids.map((bid, index) => {
        const bidder = bid.info.bidderPubkey.toBase58();
        return (
          <Row key={index}>
            <Col span={1}>{index + 1}.</Col>
            <Col span={17}>
              <Row>
                <Identicon
                  style={{
                    width: 24,
                    height: 24,
                    marginRight: 10,
                    marginTop: 2,
                  }}
                  address={bidder}
                />{' '}
                {shortenAddress(bidder)}
              </Row>
            </Col>
            <Col span={5} style={{ textAlign: 'right' }}>
              {bid.info.lastBid.toString()}
            </Col>
          </Row>
        );
      })}
    </Col>
  );
};
