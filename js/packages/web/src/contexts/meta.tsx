import {
  programIds,
  useConnection,
  decodeMetadata,
  AuctionParser,
  decodeEdition,
  decodeMasterEdition,
  Metadata,
  getMultipleAccounts,
  cache,
  MintParser,
  ParsedAccount,
  actions,
  Edition,
  MasterEdition,
  AuctionData,
  SafetyDepositBox,
  VaultKey,
  decodeSafetyDeposit,
  BidderMetadata,
  BidderMetadataParser,
  BidderPot,
  BidderPotParser,
  BIDDER_METADATA_LEN,
  BIDDER_POT_LEN,
  decodeVault,
  Vault,
} from '@oyster/common';
import { MintInfo } from '@solana/spl-token';
import { NAME_PROGRAM_ID, VERIFICATION_AUTHORITY_OFFSET, TWITTER_VERIFICATION_AUTHORITY, TWITTER_ACCOUNT_LENGTH, NameRegistryState } from '@solana/spl-name-service';
import { Connection, PublicKey, PublicKeyAndAccount } from '@solana/web3.js';
import BN from 'bn.js';
import React, { useContext, useEffect, useMemo, useState } from 'react';
import {
  AuctionManager,
  BidRedemptionTicket,
  decodeAuctionManager,
  decodeBidRedemptionTicket,
  decodeStore,
  decodeWhitelistedCreator,
  getWhitelistedCreator,
  MetaplexKey,
  Store,
  WhitelistedCreator,
  WhitelistedCreatorParser,
} from '../models/metaplex';
import names from './../config/userNames.json';

const { MetadataKey } = actions;
export interface MetaContextState {
  metadata: ParsedAccount<Metadata>[];
  metadataByMint: Record<string, ParsedAccount<Metadata>>;
  metadataByMasterEdition: Record<string, ParsedAccount<Metadata>>;
  editions: Record<string, ParsedAccount<Edition>>;
  masterEditions: Record<string, ParsedAccount<MasterEdition>>;
  masterEditionsByMasterMint: Record<string, ParsedAccount<MasterEdition>>;
  auctionManagersByAuction: Record<string, ParsedAccount<AuctionManager>>;
  auctions: Record<string, ParsedAccount<AuctionData>>;
  vaults: Record<string, ParsedAccount<Vault>>;
  store: ParsedAccount<Store> | null;
  bidderMetadataByAuctionAndBidder: Record<
    string,
    ParsedAccount<BidderMetadata>
  >;
  safetyDepositBoxesByVaultAndIndex: Record<
    string,
    ParsedAccount<SafetyDepositBox>
  >;
  bidderPotsByAuctionAndBidder: Record<string, ParsedAccount<BidderPot>>;
  bidRedemptions: Record<string, ParsedAccount<BidRedemptionTicket>>;
  whitelistedCreatorsByCreator: Record<
    string,
    ParsedAccount<WhitelistedCreator>
  >;
}

const MetaContext = React.createContext<MetaContextState>({
  metadata: [],
  metadataByMint: {},
  masterEditions: {},
  masterEditionsByMasterMint: {},
  metadataByMasterEdition: {},
  editions: {},
  auctionManagersByAuction: {},
  auctions: {},
  vaults: {},
  store: null,
  bidderMetadataByAuctionAndBidder: {},
  safetyDepositBoxesByVaultAndIndex: {},
  bidderPotsByAuctionAndBidder: {},
  bidRedemptions: {},
  whitelistedCreatorsByCreator: {},
});

export function MetaProvider({ children = null as any }) {
  const connection = useConnection();
  const PROGRAM_IDS = programIds();

  const [metadata, setMetadata] = useState<ParsedAccount<Metadata>[]>([]);
  const [metadataByMint, setMetadataByMint] = useState<
    Record<string, ParsedAccount<Metadata>>
  >({});
  const [masterEditions, setMasterEditions] = useState<
    Record<string, ParsedAccount<MasterEdition>>
  >({});

  const [masterEditionsByMasterMint, setMasterEditionsByMasterMint] = useState<
    Record<string, ParsedAccount<MasterEdition>>
  >({});

  const [metadataByMasterEdition, setMetadataByMasterEdition] = useState<
    Record<string, ParsedAccount<Metadata>>
  >({});

  const [editions, setEditions] = useState<
    Record<string, ParsedAccount<Edition>>
  >({});
  const [auctionManagersByAuction, setAuctionManagersByAuction] = useState<
    Record<string, ParsedAccount<AuctionManager>>
  >({});

  const [bidRedemptions, setBidRedemptions] = useState<
    Record<string, ParsedAccount<BidRedemptionTicket>>
  >({});
  const [auctions, setAuctions] = useState<
    Record<string, ParsedAccount<AuctionData>>
  >({});
  const [vaults, setVaults] = useState<Record<string, ParsedAccount<Vault>>>(
    {},
  );
  const [store, setStore] = useState<ParsedAccount<Store> | null>(null);
  const [whitelistedCreatorsByCreator, setWhitelistedCreatorsByCreator] =
    useState<Record<string, ParsedAccount<WhitelistedCreator>>>({});

  const [
    bidderMetadataByAuctionAndBidder,
    setBidderMetadataByAuctionAndBidder,
  ] = useState<Record<string, ParsedAccount<BidderMetadata>>>({});
  const [bidderPotsByAuctionAndBidder, setBidderPotsByAuctionAndBidder] =
    useState<Record<string, ParsedAccount<BidderPot>>>({});
  const [
    safetyDepositBoxesByVaultAndIndex,
    setSafetyDepositBoxesByVaultAndIndex,
  ] = useState<Record<string, ParsedAccount<SafetyDepositBox>>>({});

  useEffect(() => {});

  useEffect(() => {
    let dispose = () => {};
    (async () => {
      const processAuctions = async (a: PublicKeyAndAccount<Buffer>) => {
        try {
          const account = cache.add(
            a.pubkey,
            a.account,
            AuctionParser,
          ) as ParsedAccount<AuctionData>;

          setAuctions(e => ({
            ...e,
            [a.pubkey.toBase58()]: account,
          }));
        } catch {
          // ignore errors
          // add type as first byte for easier deserialization
        }

        try {
          if (a.account.data.length === BIDDER_METADATA_LEN) {
            const account = cache.add(
              a.pubkey,
              a.account,
              BidderMetadataParser,
            ) as ParsedAccount<BidderMetadata>;
            setBidderMetadataByAuctionAndBidder(e => ({
              ...e,
              [account.info.auctionPubkey.toBase58() +
              '-' +
              account.info.bidderPubkey.toBase58()]: account,
            }));
          }
        } catch {
          // ignore errors
          // add type as first byte for easier deserialization
        }
        try {
          if (a.account.data.length === BIDDER_POT_LEN) {
            const account = cache.add(
              a.pubkey,
              a.account,
              BidderPotParser,
            ) as ParsedAccount<BidderPot>;

            setBidderPotsByAuctionAndBidder(e => ({
              ...e,
              [account.info.auctionAct.toBase58() +
              '-' +
              account.info.bidderAct.toBase58()]: account,
            }));
          }
        } catch {
          // ignore errors
          // add type as first byte for easier deserialization
        }
      };

      const accounts = await connection.getProgramAccounts(
        programIds().auction,
      );
      for (let i = 0; i < accounts.length; i++) {
        await processAuctions(accounts[i]);
      }

      let subId = connection.onProgramAccountChange(
        programIds().auction,
        async info => {
          const pubkey =
            typeof info.accountId === 'string'
              ? new PublicKey(info.accountId as unknown as string)
              : info.accountId;
          await processAuctions({
            pubkey,
            account: info.accountInfo,
          });
        },
      );
      dispose = () => {
        connection.removeProgramAccountChangeListener(subId);
      };
    })();

    return () => {
      dispose();
    };
  }, [connection, setAuctions]);

  useEffect(() => {
    let dispose = () => {};
    (async () => {
      const processVaultData = async (a: PublicKeyAndAccount<Buffer>) => {
        try {
          if (a.account.data[0] === VaultKey.SafetyDepositBoxV1) {
            const safetyDeposit = await decodeSafetyDeposit(a.account.data);
            const account: ParsedAccount<SafetyDepositBox> = {
              pubkey: a.pubkey,
              account: a.account,
              info: safetyDeposit,
            };
            setSafetyDepositBoxesByVaultAndIndex(e => ({
              ...e,
              [safetyDeposit.vault.toBase58() + '-' + safetyDeposit.order]:
                account,
            }));
          } else if (a.account.data[0] === VaultKey.VaultV1) {
            const vault = await decodeVault(a.account.data);
            const account: ParsedAccount<Vault> = {
              pubkey: a.pubkey,
              account: a.account,
              info: vault,
            };
            setVaults(e => ({
              ...e,
              [a.pubkey.toBase58()]: account,
            }));
          }
        } catch {
          // ignore errors
          // add type as first byte for easier deserialization
        }
      };

      const accounts = await connection.getProgramAccounts(programIds().vault);
      for (let i = 0; i < accounts.length; i++) {
        await processVaultData(accounts[i]);
      }

      let subId = connection.onProgramAccountChange(
        programIds().vault,
        async info => {
          const pubkey =
            typeof info.accountId === 'string'
              ? new PublicKey(info.accountId as unknown as string)
              : info.accountId;
          await processVaultData({
            pubkey,
            account: info.accountInfo,
          });
        },
      );
      dispose = () => {
        connection.removeProgramAccountChangeListener(subId);
      };
    })();

    return () => {
      dispose();
    };
  }, [connection, setSafetyDepositBoxesByVaultAndIndex, setVaults]);

  useEffect(() => {
    let dispose = () => {};
    (async () => {
      const processMetaplexAccounts = async (
        a: PublicKeyAndAccount<Buffer>,
      ) => {
        try {
          if (a.account.data[0] === MetaplexKey.AuctionManagerV1) {
            const storeKey = new PublicKey(a.account.data.slice(1, 33));
            if (storeKey.toBase58() == PROGRAM_IDS.store.toBase58()) {
              const auctionManager = await decodeAuctionManager(a.account.data);
              const account: ParsedAccount<AuctionManager> = {
                pubkey: a.pubkey,
                account: a.account,
                info: auctionManager,
              };
              setAuctionManagersByAuction(e => ({
                ...e,
                [auctionManager.auction.toBase58()]: account,
              }));
            }
          } else if (a.account.data[0] === MetaplexKey.BidRedemptionTicketV1) {
            const ticket = await decodeBidRedemptionTicket(a.account.data);
            const account: ParsedAccount<BidRedemptionTicket> = {
              pubkey: a.pubkey,
              account: a.account,
              info: ticket,
            };
            setBidRedemptions(e => ({
              ...e,
              [a.pubkey.toBase58()]: account,
            }));
          } else if (a.account.data[0] == MetaplexKey.StoreV1) {
            const store = await decodeStore(a.account.data);
            const account: ParsedAccount<Store> = {
              pubkey: a.pubkey,
              account: a.account,
              info: store,
            };
            if (a.pubkey.toBase58() == PROGRAM_IDS.store.toBase58())
              setStore(account);
          } else if (a.account.data[0] == MetaplexKey.WhitelistedCreatorV1) {
            const whitelistedCreator = await decodeWhitelistedCreator(
              a.account.data,
            );
            const creatorKeyIfCreatorWasPartOfThisStore =
              await getWhitelistedCreator(whitelistedCreator.address);
            if (
              creatorKeyIfCreatorWasPartOfThisStore.toBase58() ==
              a.pubkey.toBase58()
            ) {
              const account = cache.add(
                a.pubkey,
                a.account,
                WhitelistedCreatorParser,
              ) as ParsedAccount<WhitelistedCreator>;

              const nameInfo = (names as any)[account.info.address.toBase58()];
              if(nameInfo) {
                account.info.name = nameInfo.name;
                account.info.image = nameInfo.image;
                account.info.twitter = nameInfo.twitter;
              }

              setWhitelistedCreatorsByCreator(e => ({
                ...e,
                [whitelistedCreator.address.toBase58()]: account,
              }));
            }
          }
        } catch {
          // ignore errors
          // add type as first byte for easier deserialization
        }
      };

      const accounts = await connection.getProgramAccounts(
        programIds().metaplex,
      );
      for (let i = 0; i < accounts.length; i++) {
        await processMetaplexAccounts(accounts[i]);
      }

      let subId = connection.onProgramAccountChange(
        programIds().metaplex,
        async info => {
          const pubkey =
            typeof info.accountId === 'string'
              ? new PublicKey(info.accountId as unknown as string)
              : info.accountId;
          await processMetaplexAccounts({
            pubkey,
            account: info.accountInfo,
          });
        },
      );
      dispose = () => {
        connection.removeProgramAccountChangeListener(subId);
      };
    })();

    return () => {
      dispose();
    };
  }, [connection, setAuctionManagersByAuction, setBidRedemptions]);

  useEffect(() => {
    let dispose = () => {};
    (async () => {
      const processMetaData = async (meta: PublicKeyAndAccount<Buffer>) => {
        try {
          if (meta.account.data[0] === MetadataKey.MetadataV1) {
            const metadata = await decodeMetadata(meta.account.data);
            if (
              isValidHttpUrl(metadata.data.uri) &&
              metadata.data.uri.indexOf('arweave') >= 0
            ) {
              const account: ParsedAccount<Metadata> = {
                pubkey: meta.pubkey,
                account: meta.account,
                info: metadata,
              };
              setMetadataByMint(e => ({
                ...e,
                [metadata.mint.toBase58()]: account,
              }));
              setMetadataByMasterEdition(e => ({
                ...e,
                [metadata.masterEdition?.toBase58() || '']: account,
              }));
            }
          } else if (meta.account.data[0] === MetadataKey.EditionV1) {
            const edition = decodeEdition(meta.account.data);
            const account: ParsedAccount<Edition> = {
              pubkey: meta.pubkey,
              account: meta.account,
              info: edition,
            };
            setEditions(e => ({ ...e, [meta.pubkey.toBase58()]: account }));
          } else if (meta.account.data[0] === MetadataKey.MasterEditionV1) {
            const masterEdition = decodeMasterEdition(meta.account.data);
            const account: ParsedAccount<MasterEdition> = {
              pubkey: meta.pubkey,
              account: meta.account,
              info: masterEdition,
            };
            setMasterEditions(e => ({
              ...e,
              [meta.pubkey.toBase58()]: account,
            }));
            setMasterEditionsByMasterMint(e => ({
              ...e,
              [masterEdition.masterMint.toBase58()]: account,
            }));
          }
        } catch {
          // ignore errors
          // add type as first byte for easier deserialization
        }
      };

      const accounts = await connection.getProgramAccounts(
        programIds().metadata,
      );
      for (let i = 0; i < accounts.length; i++) {
        await processMetaData(accounts[i]);
      }

      setMetadataByMint(latest => {
        queryExtendedMetadata(
          connection,
          setMetadata,
          setMetadataByMint,
          latest,
        );
        return latest;
      });

      let subId = connection.onProgramAccountChange(
        programIds().metadata,
        async info => {
          const pubkey =
            typeof info.accountId === 'string'
              ? new PublicKey(info.accountId as unknown as string)
              : info.accountId;
          await processMetaData({
            pubkey,
            account: info.accountInfo,
          });
          setMetadataByMint(latest => {
            queryExtendedMetadata(
              connection,
              setMetadata,
              setMetadataByMint,
              latest,
            );
            return latest;
          });
        },
      );
      dispose = () => {
        connection.removeProgramAccountChangeListener(subId);
      };
    })();

    return () => {
      dispose();
    };
  }, [
    connection,
    setMetadata,
    setMasterEditions,
    setMasterEditionsByMasterMint,
    setMetadataByMasterEdition,
    setEditions,
  ]);
  const memoizedMeta = useMemo(
    () =>
      metadata.filter(m =>
        m.info.data.creators?.find(
          c =>
            c.verified &&
            whitelistedCreatorsByCreator[c.address.toBase58()].info.activated,
        ),
      ),
    [metadata, whitelistedCreatorsByCreator],
  );

  useEffect(() => {
    // TODO: fetch names dynamically

  })

  // TODO: get names for creators
  // useEffect(() => {
  //   (async () => {
  //     const twitterHandles = await connection.getProgramAccounts(NAME_PROGRAM_ID, {
  //      filters: [
  //        {
  //           dataSize: TWITTER_ACCOUNT_LENGTH,
  //        },
  //        {
  //          memcmp: {
  //           offset: VERIFICATION_AUTHORITY_OFFSET,
  //           bytes: TWITTER_VERIFICATION_AUTHORITY.toBase58()
  //          }
  //        }
  //      ]
  //     });

  //     const handles = twitterHandles.map(t => {
  //       const owner = new PublicKey(t.account.data.slice(32, 64));
  //       const name = t.account.data.slice(96, 114).toString();
  //     });


  //     console.log(handles);

  //   })();
  // }, [whitelistedCreatorsByCreator]);

  return (
    <MetaContext.Provider
      value={{
        metadata: memoizedMeta,
        editions,
        masterEditions,
        auctionManagersByAuction,
        auctions,
        metadataByMint,
        safetyDepositBoxesByVaultAndIndex,
        bidderMetadataByAuctionAndBidder,
        bidderPotsByAuctionAndBidder,
        vaults,
        bidRedemptions,
        masterEditionsByMasterMint,
        metadataByMasterEdition,
        whitelistedCreatorsByCreator,
        store,
      }}
    >
      {children}
    </MetaContext.Provider>
  );
}

const queryExtendedMetadata = async (
  connection: Connection,
  setMetadata: (metadata: ParsedAccount<Metadata>[]) => void,
  setMetadataByMint: (
    metadata: Record<string, ParsedAccount<Metadata>>,
  ) => void,
  mintToMeta: Record<string, ParsedAccount<Metadata>>,
) => {
  const mintToMetadata = { ...mintToMeta };

  const mints = await getMultipleAccounts(
    connection,
    [...Object.keys(mintToMetadata)].filter(k => !cache.get(k)),
    'single',
  );
  mints.keys.forEach((key, index) => {
    const mintAccount = mints.array[index];
    const mint = cache.add(
      key,
      mintAccount,
      MintParser,
    ) as ParsedAccount<MintInfo>;
    if (mint.info.supply.gt(new BN(1)) || mint.info.decimals !== 0) {
      // naive not NFT check
      delete mintToMetadata[key];
    } else {
      // const metadata = mintToMetadata[key];
    }
  });

  // await Promise.all([...extendedMetadataFetch.values()]);

  setMetadata([...Object.values(mintToMetadata)]);
  setMetadataByMint(mintToMetadata);
};

export const useMeta = () => {
  const context = useContext(MetaContext);
  return context as MetaContextState;
};

function isValidHttpUrl(text: string) {
  let url;

  try {
    url = new URL(text);
  } catch (_) {
    return false;
  }

  return url.protocol === 'http:' || url.protocol === 'https:';
}
