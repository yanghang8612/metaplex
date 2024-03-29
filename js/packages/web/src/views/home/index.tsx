import React, { useState } from 'react';
import { Layout, Row, Col, Tabs } from 'antd';
import Masonry from 'react-masonry-css';

import { PreSaleBanner } from '../../components/PreSaleBanner';
import { AuctionViewState, useAuctions } from '../../hooks';

import './index.less';
import { ArtCard } from '../../components/ArtCard';
import { Link } from 'react-router-dom';
import { EditionType } from '../../models/metaplex';

const { TabPane } = Tabs;

const { Content } = Layout;
export const HomeView = () => {
  const [activeKey, setActiveKey] = useState(AuctionViewState.Live);
  const auctions = useAuctions(activeKey);
  const breakpointColumnsObj = {
    default: 4,
    1100: 3,
    700: 2,
    500: 1,
  };

  const auctionGrid = (
    <Masonry
      breakpointCols={breakpointColumnsObj}
      className="my-masonry-grid"
      columnClassName="my-masonry-grid_column"
    >
      {auctions.map((m, idx) => {
        const id = m.auction.pubkey.toBase58();
        const winningConfig = m.auctionManager.info.settings.winningConfigs.find(
          w => w.safetyDepositBoxIndex === m.thumbnail.safetyDeposit.info.order,
        );
        return (
          <Link to={`/auction/${id}`} key={idx}>
            <ArtCard
              key={id}
              endAuctionAt={m.auction.info.endAuctionAt?.toNumber()}
              pubkey={m.thumbnail.metadata.pubkey}
              editionType={
                winningConfig
                  ? winningConfig.editionType
                  : EditionType.OpenEdition
              }
              preview={false}
            />
          </Link>
        );
      })}
    </Masonry>
  );

  return (
    <Layout style={{ margin: 0, marginTop: 30 }}>
      <PreSaleBanner
        artistName={'RAC'}
        productName={'THE BOY COLLECTION'}
        preSaleTS={1620009209}
        image="img/banner1.jpeg"
      />
      <Layout>
        <Content style={{ display: 'flex', flexWrap: 'wrap' }}>
          <Col style={{ width: '100%', marginTop: 10 }}>
            <Row>
              <Tabs
                activeKey={activeKey}
                onTabClick={key => setActiveKey(key as AuctionViewState)}
              >
                <TabPane
                  tab={<span className="tab-title">Live</span>}
                  key={AuctionViewState.Live}
                >
                  {auctionGrid}
                </TabPane>
                <TabPane
                  tab={<span className="tab-title">Upcoming</span>}
                  key={AuctionViewState.Upcoming}
                >
                  {auctionGrid}
                </TabPane>
                <TabPane
                  tab={<span className="tab-title">Ended</span>}
                  key={AuctionViewState.Ended}
                >
                  {auctionGrid}
                </TabPane>
                <TabPane
                  tab={<span className="tab-title">Buy Now</span>}
                  key={AuctionViewState.BuyNow}
                >
                  {auctionGrid}
                </TabPane>
              </Tabs>
            </Row>
          </Col>
        </Content>
      </Layout>
    </Layout>
  );
};
