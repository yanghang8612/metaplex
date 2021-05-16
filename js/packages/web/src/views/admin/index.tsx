import React, { useState } from 'react';
import { Layout, Row, Col, Table, Switch, Spin } from 'antd';
import { useMeta } from '../../contexts';
import { Store, WhitelistedCreator } from '../../models/metaplex';
import { ParsedAccount } from '@oyster/common';
import { ARTISTS } from '../../constants/artists';

const { Content } = Layout;
export const AdminView = () => {
  const { store, whitelistedCreators } = useMeta();

  return store ? (
    <InnerAdminView store={store} whitelistedCreators={whitelistedCreators} />
  ) : (
    <Spin />
  );
};

function InnerAdminView({
  store,
  whitelistedCreators,
}: {
  store: ParsedAccount<Store>;
  whitelistedCreators: Record<string, ParsedAccount<WhitelistedCreator>>;
}) {
  const [newStore, setNewStore] = useState(new Store(store.info));
  const [updatedCreators, setUpdatedCreators] = useState<
    Record<string, WhitelistedCreator>
  >({});

  const uniqueCreators = Object.values(whitelistedCreators).reduce(
    (acc: Record<string, WhitelistedCreator>, e) => {
      acc[e.info.address.toBase58()] = e.info;
      return acc;
    },
    {},
  );

  const uniqueCreatorsWithUpdates = { ...uniqueCreators, ...updatedCreators };

  const columns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: 'Address',
      dataIndex: 'address',
      key: 'address',
    },
    {
      title: 'Activated',
      dataIndex: 'activated',
      key: 'activated',
    },
  ];

  return (
    <Layout style={{ margin: 0, marginTop: 30 }}>
      <Content style={{ display: 'flex', flexWrap: 'wrap' }}>
        <Col style={{ width: '100%', marginTop: 10 }}>
          <Row>
            <Switch
              checkedChildren="Public"
              unCheckedChildren="Whitelist Only"
              checked={newStore.public}
              onChange={val => {
                setNewStore(store => {
                  store.public = val;
                  return store;
                });
              }}
            />
          </Row>
          <Row>
            <Table
              columns={columns}
              dataSource={Object.keys(uniqueCreatorsWithUpdates).map(key => ({
                key,
                address: uniqueCreatorsWithUpdates[key].address,
                activated: uniqueCreatorsWithUpdates[key].activated,
                name: ARTISTS.find(
                  a =>
                    a.address.toBase58() ==
                    uniqueCreatorsWithUpdates[key].address.toBase58(),
                )?.name,
              }))}
            ></Table>
          </Row>
        </Col>
      </Content>
    </Layout>
  );
}
