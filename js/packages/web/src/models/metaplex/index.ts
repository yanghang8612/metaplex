import {
  AUCTION_PREFIX,
  programIds,
  METADATA,
  AccountParser,
} from '@oyster/common';
import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { deserializeUnchecked } from 'borsh';

export * from './initAuctionManager';
export * from './redeemBid';
export * from './redeemMasterEditionBid';
export * from './redeemOpenEditionBid';
export * from './startAuction';
export * from './validateSafetyDepositBox';

export const METAPLEX_PREFIX = 'metaplex';
export const ORIGINAL_AUTHORITY_LOOKUP_SIZE = 33;

export enum MetaplexKey {
  AuctionManagerV1 = 0,
  OriginalAuthorityLookupV1 = 1,
  BidRedemptionTicketV1 = 2,
  StoreV1 = 3,
  WhitelistedCreatorV1 = 4,
}
export class AuctionManager {
  key: MetaplexKey;
  store: PublicKey;
  authority: PublicKey;
  auction: PublicKey;
  vault: PublicKey;
  acceptPayment: PublicKey;
  state: AuctionManagerState;
  settings: AuctionManagerSettings;

  constructor(args: {
    store: PublicKey;
    authority: PublicKey;
    auction: PublicKey;
    vault: PublicKey;
    acceptPayment: PublicKey;
    state: AuctionManagerState;
    settings: AuctionManagerSettings;
  }) {
    this.key = MetaplexKey.AuctionManagerV1;
    this.store = args.store;
    this.authority = args.authority;
    this.auction = args.auction;
    this.vault = args.vault;
    this.acceptPayment = args.acceptPayment;
    this.state = args.state;
    this.settings = args.settings;
  }
}

export class InitAuctionManagerArgs {
  instruction = 0;
  settings: AuctionManagerSettings;

  constructor(args: { settings: AuctionManagerSettings }) {
    this.settings = args.settings;
  }
}

export class ValidateSafetyDepositBoxArgs {
  instruction = 1;
}

export class RedeemBidArgs {
  instruction = 2;
}

export class RedeemMasterEditionBidArgs {
  instruction = 3;
}

export class RedeemOpenEditionBidArgs {
  instruction = 4;
}

export class StartAuctionArgs {
  instruction = 5;
}
export class ClaimBidArgs {
  instruction = 6;
}
export class EmptyPaymentAccountArgs {
  instruction = 7;
}

export class SetStoreArgs {
  instruction = 8;
  public: boolean;
  constructor(args: { public: boolean }) {
    this.public = args.public;
  }
}

export class SetWhitelistedCreatorArgs {
  instruction = 9;
  activated: boolean;
  constructor(args: { activated: boolean }) {
    this.activated = args.activated;
  }
}

export class ValidateOpenEditionArgs {
  instruction = 10;
}

export enum WinningConstraint {
  NoOpenEdition = 0,
  OpenEditionGiven = 1,
}

export enum NonWinningConstraint {
  NoOpenEdition = 0,
  GivenForFixedPrice = 1,
  GivenForBidPrice = 2,
}

export class AuctionManagerSettings {
  openEditionWinnerConstraint: WinningConstraint =
    WinningConstraint.NoOpenEdition;
  openEditionNonWinningConstraint: NonWinningConstraint =
    NonWinningConstraint.GivenForFixedPrice;
  winningConfigs: WinningConfig[] = [];
  openEditionConfig: number | null = 0;
  openEditionFixedPrice: BN | null = new BN(0);

  constructor(args?: AuctionManagerSettings) {
    Object.assign(this, args);
  }
}

export enum EditionType {
  // Not an edition
  NA,
  /// Means you are auctioning off the master edition record
  MasterEdition,
  /// Means you are using authorization tokens to print off limited editions during the auction
  LimitedEdition,
  /// Means you are using the master edition to print off new editions during the auction
  OpenEdition,
}

export class WinningConfig {
  safetyDepositBoxIndex: number = 0;
  amount: number = 0;
  editionType: EditionType = EditionType.NA;

  constructor(args?: WinningConfig) {
    Object.assign(this, args);
  }
}

export const decodeWhitelistedCreator = (buffer: Buffer) => {
  return deserializeUnchecked(
    SCHEMA,
    WhitelistedCreator,
    buffer,
  ) as WhitelistedCreator;
};

export const WhitelistedCreatorParser: AccountParser = (
  pubkey: PublicKey,
  account: AccountInfo<Buffer>,
) => ({
  pubkey,
  account,
  info: decodeWhitelistedCreator(account.data),
});

export const decodeStore = (buffer: Buffer) => {
  return deserializeUnchecked(SCHEMA, Store, buffer) as Store;
};

export const decodeAuctionManager = (buffer: Buffer) => {
  return deserializeUnchecked(SCHEMA, AuctionManager, buffer) as AuctionManager;
};

export const decodeBidRedemptionTicket = (buffer: Buffer) => {
  return deserializeUnchecked(
    SCHEMA,
    BidRedemptionTicket,
    buffer,
  ) as BidRedemptionTicket;
};

export class WinningConfigState {
  amountMinted: number = 0;
  validated: boolean = false;
  claimed: boolean = false;

  constructor(args?: WinningConfigState) {
    Object.assign(this, args);
  }
}

export class WhitelistedCreator {
  key: MetaplexKey = MetaplexKey.WhitelistedCreatorV1;
  address: PublicKey;
  activated: boolean = true;

  // Populated from name service
  twitter?: string;
  name?: string;
  image?: string;

  constructor(args: { address: PublicKey; activated: boolean }) {
    this.address = args.address;
    this.activated = args.activated;
  }
}

export class Store {
  key: MetaplexKey = MetaplexKey.StoreV1;
  public: boolean = true;
  auctionProgram: PublicKey;
  tokenVaultProgram: PublicKey;
  tokenMetadataProgram: PublicKey;
  tokenProgram: PublicKey;

  constructor(args: {
    public: boolean;
    auctionProgram: PublicKey;
    tokenVaultProgram: PublicKey;
    tokenMetadataProgram: PublicKey;
    tokenProgram: PublicKey;
  }) {
    this.key = MetaplexKey.StoreV1;
    this.public = args.public;
    this.auctionProgram = args.auctionProgram;
    this.tokenVaultProgram = args.tokenVaultProgram;
    this.tokenMetadataProgram = args.tokenMetadataProgram;
    this.tokenProgram = args.tokenProgram;
  }
}

export class AuctionManagerState {
  status: AuctionManagerStatus = AuctionManagerStatus.Initialized;
  winningConfigsValidated: number = 0;
  masterEditionsWithAuthoritiesRemainingToReturn: number = 0;

  winningConfigStates: WinningConfigState[] = [];

  constructor(args?: AuctionManagerState) {
    Object.assign(this, args);
  }
}

export enum AuctionManagerStatus {
  Initialized,
  Validated,
  Running,
  Disbursing,
  Finished,
}

export class BidRedemptionTicket {
  key: MetaplexKey = MetaplexKey.BidRedemptionTicketV1;
  openEditionRedeemed: boolean = false;
  bidRedeemed: boolean = false;

  constructor(args?: BidRedemptionTicket) {
    Object.assign(this, args);
  }
}

export const SCHEMA = new Map<any, any>([
  [
    AuctionManager,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['store', 'pubkey'],
        ['authority', 'pubkey'],
        ['auction', 'pubkey'],
        ['vault', 'pubkey'],
        ['acceptPayment', 'pubkey'],
        ['state', AuctionManagerState],
        ['settings', AuctionManagerSettings],
      ],
    },
  ],
  [
    AuctionManagerSettings,
    {
      kind: 'struct',
      fields: [
        ['openEditionWinnerConstraint', 'u8'], // enum
        ['openEditionNonWinningConstraint', 'u8'],
        ['winningConfigs', [WinningConfig]],
        ['openEditionConfig', { kind: 'option', type: 'u8' }],
        ['openEditionFixedPrice', { kind: 'option', type: 'u64' }],
      ],
    },
  ],
  [
    WinningConfig,
    {
      kind: 'struct',
      fields: [
        ['safetyDepositBoxIndex', 'u8'],
        ['amount', 'u8'],
        ['editionType', 'u8'],
      ],
    },
  ],
  [
    WinningConfigState,
    {
      kind: 'struct',
      fields: [
        ['amountMinted', 'u8'],
        ['validated', 'u8'], // bool
        ['claimed', 'u8'], // bool
      ],
    },
  ],
  [
    WhitelistedCreator,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['address', 'pubkey'],
        ['activated', 'u8'],
      ],
    },
  ],
  [
    Store,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['public', 'u8'],
        ['auctionProgram', 'pubkey'],
        ['tokenVaultProgram', 'pubkey'],
        ['tokenMetadataProgram', 'pubkey'],
        ['tokenProgram', 'pubkey'],
      ],
    },
  ],
  [
    AuctionManagerState,
    {
      kind: 'struct',
      fields: [
        ['status', 'u8'],
        ['winningConfigsValidated', 'u8'],
        ['masterEditionsWithAuthoritiesRemainingToReturn', 'u8'],
        ['winningConfigStates', [WinningConfigState]],
      ],
    },
  ],
  [
    BidRedemptionTicket,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['openEditionRedeemed', 'u8'], // bool
        ['bidRedeemed', 'u8'], // bool
      ],
    },
  ],
  [
    InitAuctionManagerArgs,
    {
      kind: 'struct',
      fields: [
        ['instruction', 'u8'],
        ['settings', AuctionManagerSettings],
      ],
    },
  ],
  [
    ValidateSafetyDepositBoxArgs,
    {
      kind: 'struct',
      fields: [['instruction', 'u8']],
    },
  ],
  [
    RedeemBidArgs,
    {
      kind: 'struct',
      fields: [['instruction', 'u8']],
    },
  ],
  [
    RedeemMasterEditionBidArgs,
    {
      kind: 'struct',
      fields: [['instruction', 'u8']],
    },
  ],
  [
    RedeemOpenEditionBidArgs,
    {
      kind: 'struct',
      fields: [['instruction', 'u8']],
    },
  ],
  [
    StartAuctionArgs,
    {
      kind: 'struct',
      fields: [['instruction', 'u8']],
    },
  ],
  [
    ClaimBidArgs,
    {
      kind: 'struct',
      fields: [['instruction', 'u8']],
    },
  ],
  [
    EmptyPaymentAccountArgs,
    {
      kind: 'struct',
      fields: [['instruction', 'u8']],
    },
  ],
  [
    SetStoreArgs,
    {
      kind: 'struct',
      fields: [
        ['instruction', 'u8'],
        ['public', 'u8'], //bool
      ],
    },
  ],
  [
    SetWhitelistedCreatorArgs,
    {
      kind: 'struct',
      fields: [
        ['instruction', 'u8'],
        ['activated', 'u8'], //bool
      ],
    },
  ],
  [
    ValidateOpenEditionArgs,
    {
      kind: 'struct',
      fields: [['instruction', 'u8']],
    },
  ],
]);

export async function getAuctionManagerKey(
  vault: PublicKey,
  auctionKey: PublicKey,
): Promise<PublicKey> {
  const PROGRAM_IDS = programIds();

  return (
    await PublicKey.findProgramAddress(
      [Buffer.from(METAPLEX_PREFIX), auctionKey.toBuffer()],
      PROGRAM_IDS.metaplex,
    )
  )[0];
}

export async function getAuctionKeys(
  vault: PublicKey,
): Promise<{ auctionKey: PublicKey; auctionManagerKey: PublicKey }> {
  const PROGRAM_IDS = programIds();

  const auctionKey: PublicKey = (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(AUCTION_PREFIX),
        PROGRAM_IDS.auction.toBuffer(),
        vault.toBuffer(),
      ],
      PROGRAM_IDS.auction,
    )
  )[0];

  const auctionManagerKey = await getAuctionManagerKey(vault, auctionKey);

  return { auctionKey, auctionManagerKey };
}

export async function getBidderKeys(
  auctionKey: PublicKey,
  bidder: PublicKey,
): Promise<{ bidMetadata: PublicKey; bidRedemption: PublicKey }> {
  const PROGRAM_IDS = programIds();

  const bidMetadata: PublicKey = (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(AUCTION_PREFIX),
        PROGRAM_IDS.auction.toBuffer(),
        auctionKey.toBuffer(),
        bidder.toBuffer(),
        Buffer.from(METADATA),
      ],
      PROGRAM_IDS.auction,
    )
  )[0];

  const bidRedemption: PublicKey = (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(METAPLEX_PREFIX),
        auctionKey.toBuffer(),
        bidMetadata.toBuffer(),
      ],
      PROGRAM_IDS.metaplex,
    )
  )[0];

  return { bidMetadata, bidRedemption };
}

export async function getOriginalAuthority(
  auctionKey: PublicKey,
  metadata: PublicKey,
): Promise<PublicKey> {
  const PROGRAM_IDS = programIds();

  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(METAPLEX_PREFIX),
        auctionKey.toBuffer(),
        metadata.toBuffer(),
      ],
      PROGRAM_IDS.metaplex,
    )
  )[0];
}

export async function getWhitelistedCreator(creator: PublicKey) {
  const PROGRAM_IDS = programIds();
  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(METAPLEX_PREFIX),
        PROGRAM_IDS.metaplex.toBuffer(),
        PROGRAM_IDS.store.toBuffer(),
        creator.toBuffer(),
      ],
      PROGRAM_IDS.metaplex,
    )
  )[0];
}
