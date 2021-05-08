import React, { useEffect, useState } from 'react';
import { Card, Avatar, CardProps, Button, Badge } from 'antd';
import { MetadataCategory } from '@oyster/common';
import { ArtContent } from './../ArtContent';
import './index.less';
import { getCountdown } from '../../utils/utils';
import { useArt } from '../../hooks';
import { PublicKey } from '@solana/web3.js';
import { ArtType } from '../../types';
import { EditionType } from '../../models/metaplex';

const { Meta } = Card;

export interface ArtCardProps extends CardProps {
  pubkey?: PublicKey;
  image?: string;
  category?: MetadataCategory;
  name?: string;
  symbol?: string;
  description?: string;
  artist?: string;
  preview?: boolean;
  small?: boolean;
  close?: () => void;
  editionType?: EditionType;
  endAuctionAt?: number;
}

export const ArtCard = (props: ArtCardProps) => {
  let {
    className,
    small,
    category,
    image,
    name,
    preview,
    artist,
    description,
    close,
    pubkey,
    endAuctionAt,
    editionType,
    ...rest
  } = props;
  const art = useArt(pubkey);
  category = art?.category || category;
  image = art?.image || image;
  name = art?.title || name || '';
  artist = art?.artist || artist;
  description = art?.about || description;

  const [hours, setHours] = useState<number>(0);
  const [minutes, setMinutes] = useState<number>(0);
  const [seconds, setSeconds] = useState<number>(0);
  const [hasTimer, setHasTimer] = useState(false);

  useEffect(() => {
    const interval = setInterval(() => {
      if (!endAuctionAt) return;
      const { hours, minutes, seconds } = getCountdown(endAuctionAt);

      setHours(hours);
      setMinutes(minutes);
      setSeconds(seconds);
      setHasTimer(true);
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  let badge = '';
  if (art.type === ArtType.NFT) {
    badge = "Unique";
  } else if (art.type === ArtType.Master) {
    badge = "NFT 0";
  } else if (art.type === ArtType.Print) {
    badge = `${art.edition} of ${art.supply}`;
  }

  const card = (
    <Card
      hoverable={true}
      className={`art-card ${small ? 'small' : ''} ${className}`}
      cover={
        <>
          {close && (
            <Button
              className="card-close-button"
              shape="circle"
              onClick={e => {
                e.stopPropagation();
                e.preventDefault();
                close && close();
              }}
            >
              X
            </Button>
          )}
          <ArtContent category={category} content={image} preview={preview} />
        </>
      }
      {...rest}
    >
      <Meta
        title={`${name}`}
        description={
          <>
            <Avatar src="img/artist1.jpeg" /> {artist}
            {/* {art.type === ArtType.Master && (
              <>
                <br />
                {!endAuctionAt && (
                  <span style={{ padding: '24px' }}>
                    {(art.maxSupply || 0) - (art.supply || 0)}/
                    {art.maxSupply || 0} prints remaining
                  </span>
                )}
              </>
            )} */}

            <div className="edition-badge">{badge}</div>
            {endAuctionAt && hasTimer && (
              <div className="cd-container">
                {hours === 0 && minutes === 0 && seconds === 0 ? (
                  <div className="cd-title">Finished</div>
                ) : (
                  <>
                    <div className="cd-title">Ending in</div>
                    <div className="cd-time">
                      {hours}h {minutes}m {seconds}s
                    </div>
                  </>
                )}
              </div>
            )}
          </>
        }
      />
    </Card>
  );

  return card;
};
