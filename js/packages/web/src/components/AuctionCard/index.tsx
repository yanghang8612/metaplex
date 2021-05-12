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
  formatAmount,
} from '@oyster/common';
import { AuctionView, AuctionViewState, useBidsForAuction, useUserBalance } from '../../hooks';
import { sendPlaceBid } from '../../actions/sendPlaceBid';
import { sendRedeemBid } from '../../actions/sendRedeemBid';
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
  const [showAlert, setShowAlert] = useState<boolean>(false);
  const { accountByMint } = useUserAccounts();

  const bids = useBidsForAuction(auctionView.auction.pubkey);

  const balance = useUserBalance(auctionView.auction.info.tokenMint);

  const myPayingAccount = balance.accounts[0];

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

  useEffect(() => {
    let timeout: ReturnType<typeof setTimeout>;
    if (showAlert) {
      timeout = setTimeout(() => setShowAlert(false), 5000);
    }
    return () => clearTimeout(timeout);
  }, [showAlert]);

  const isUpcoming = auctionView.state === AuctionViewState.Upcoming;
  const isStarted = auctionView.state === AuctionViewState.Live;

  return (
    <div className="presale-card-container">
      {isUpcoming && <div className="info-header">STARTING BID</div>}
      {isUpcoming && (
        <div style={{ fontWeight: 700, fontSize: '1.6rem' }}>◎40.00</div>
      )}
      {isStarted && <div className="info-header">HIGHEST BID</div>}
      {isStarted && (
        <div style={{ fontWeight: 700, fontSize: '1.6rem' }}>◎40.00</div>
      )}
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
      />

      <div
        className="info-content"
        style={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: '0.9rem' }}
      >
        Your Balance: ◎{formatAmount(balance.balance, 2)} (${formatAmount(balance.balanceInUSD, 2)})
      </div>

      {auctionView.state === AuctionViewState.Ended ? (
        <Button
          type="primary"
          size="large"
          className="action-btn"
          disabled={!auctionView.myBidderMetadata}
          onClick={() => {
            console.log('Auctionview', auctionView);
            sendRedeemBid(connection, wallet, auctionView, accountByMint);
          }}
          style={{ marginTop: 20 }}
        >
          REDEEM BID
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
              );
              setShowAlert(true);
            }
          }}
          style={{ marginTop: 20 }}
        >
          PLACE BID
        </Button>
      )}
      <AuctionBids bids={bids} />
      {showAlert && (
        <Alert
          message="Bid placed"
          description="Congratulations! You've placed a bid successfully"
          type="success"
          showIcon
          closable
        />
      )}
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
