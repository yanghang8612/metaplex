import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import { programIds } from '../utils/ids';
import { deserializeBorsh } from './../utils/borsh';
import { serialize } from 'borsh';
import BN from 'bn.js';
export const METADATA_PREFIX = 'metadata';
export const EDITION = 'edition';

export const MAX_NAME_LENGTH = 32;

export const MAX_SYMBOL_LENGTH = 10;

export const MAX_URI_LENGTH = 200;

export const MAX_CREATOR_LIMIT = 5;

export const MAX_CREATOR_LEN = 32 + 1 + 1;

export const MAX_METADATA_LEN =
  1 +
  32 +
  32 +
  MAX_NAME_LENGTH +
  MAX_SYMBOL_LENGTH +
  MAX_URI_LENGTH +
  MAX_CREATOR_LIMIT * MAX_CREATOR_LEN +
  200;

export const MAX_MASTER_EDITION_KEN = 1 + 9 + 8 + 32;

export enum MetadataKey {
  MetadataV1 = 0,
  EditionV1 = 1,
  MasterEditionV1 = 2,
}

export enum MetadataCategory {
  Audio = 'audio',
  Video = 'video',
  Image = 'image',
}

export interface IMetadataExtension {
  name: string;
  symbol: string;
  description: string;
  // preview image
  image: string;
  // stores link to item on meta
  externalUrl: string;
  royalty: number;
  files?: File[];
  category: MetadataCategory;
  maxSupply?: number;
}

export class MasterEdition {
  key: MetadataKey;
  supply: BN;
  maxSupply?: BN;
  /// Can be used to mint tokens that give one-time permission to mint a single limited edition.
  masterMint: PublicKey;

  constructor(args: {
    key: MetadataKey;
    supply: BN;
    maxSupply?: BN;
    /// Can be used to mint tokens that give one-time permission to mint a single limited edition.
    masterMint: PublicKey;
  }) {
    this.key = MetadataKey.MasterEditionV1;
    this.supply = args.supply;
    this.maxSupply = args.maxSupply;
    this.masterMint = args.masterMint;
  }
}

export class Edition {
  key: MetadataKey;
  /// Points at MasterEdition struct
  parent: PublicKey;
  /// Starting at 0 for master record, this is incremented for each edition minted.
  edition: BN;

  constructor(args: { key: MetadataKey; parent: PublicKey; edition: BN }) {
    this.key = MetadataKey.EditionV1;
    this.parent = args.parent;
    this.edition = args.edition;
  }
}

export class Creator {
  address: PublicKey;
  verified: boolean;
  perc: number;

  constructor(args: { address: PublicKey; verified: boolean; perc: number }) {
    this.address = args.address;
    this.verified = args.verified;
    this.perc = args.perc;
  }
}

export class Data {
  name: string;
  symbol: string;
  uri: string;
  creators: Creator[] | null;
  constructor(args: {
    name: string;
    symbol: string;
    uri: string;
    creators: Creator[] | null;
  }) {
    this.name = args.name;
    this.symbol = args.symbol;
    this.uri = args.uri;
    this.creators = args.creators;
  }
}

export class Metadata {
  key: MetadataKey;
  updateAuthority: PublicKey;
  mint: PublicKey;
  data: Data;

  extended?: IMetadataExtension;
  masterEdition?: PublicKey;
  edition?: PublicKey;
  constructor(args: {
    updateAuthority: PublicKey;
    mint: PublicKey;
    data: Data;
  }) {
    this.key = MetadataKey.MetadataV1;
    this.updateAuthority = args.updateAuthority;
    this.mint = args.mint;
    this.data = args.data;
  }
}

class CreateMetadataArgs {
  instruction: number = 0;
  data: Data;

  constructor(args: { data: Data }) {
    this.data = args.data;
  }
}
class UpdateMetadataArgs {
  instruction: number = 1;
  data: Data | null;
  // Not used by this app, just required for instruction
  updateAuthority: PublicKey | null;

  constructor(args: { data?: Data; updateAuthority?: string }) {
    this.data = args.data ? args.data : null;
    this.updateAuthority = args.updateAuthority
      ? new PublicKey(args.updateAuthority)
      : null;
  }
}

class CreateMasterEditionArgs {
  instruction: number = 2;
  maxSupply: BN | null;
  constructor(args: { maxSupply: BN | null }) {
    this.maxSupply = args.maxSupply;
  }
}

export const METADATA_SCHEMA = new Map<any, any>([
  [
    CreateMetadataArgs,
    {
      kind: 'struct',
      fields: [
        ['instruction', 'u8'],
        ['data', Data],
      ],
    },
  ],
  [
    UpdateMetadataArgs,
    {
      kind: 'struct',
      fields: [
        ['instruction', 'u8'],
        ['data', { kind: 'option', type: Data }],
        ['updateAuthority', { kind: 'option', type: 'pubkey' }],
      ],
    },
  ],

  [
    CreateMasterEditionArgs,
    {
      kind: 'struct',
      fields: [
        ['instruction', 'u8'],
        ['maxSupply', { kind: 'option', type: 'u64' }],
      ],
    },
  ],
  [
    MasterEdition,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['supply', 'u64'],
        ['maxSupply', { kind: 'option', type: 'u64' }],
        ['masterMint', 'pubkey'],
      ],
    },
  ],
  [
    Edition,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['parent', 'pubkey'],
        ['edition', 'u64'],
      ],
    },
  ],
  [
    Data,
    {
      kind: 'struct',
      fields: [
        ['name', 'string'],
        ['symbol', 'string'],
        ['uri', 'string'],
        ['creators', { kind: 'option', type: [Creator] }],
      ],
    },
  ],
  [
    Creator,
    {
      kind: 'struct',
      fields: [
        ['address', 'pubkey'],
        ['verified', 'u8'],
        ['perc', 'u8'],
      ],
    },
  ],
  [
    Metadata,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['updateAuthority', 'pubkey'],
        ['mint', 'pubkey'],
        ['data', Data],
      ],
    },
  ],
]);

export const decodeMetadata = async (buffer: Buffer): Promise<Metadata> => {
  const metadata = deserializeBorsh(
    METADATA_SCHEMA,
    Metadata,
    buffer,
  ) as Metadata;
  metadata.edition = await getEdition(metadata.mint);
  metadata.masterEdition = await getEdition(metadata.mint);
  return metadata;
};

export const decodeEdition = (buffer: Buffer) => {
  return deserializeBorsh(METADATA_SCHEMA, Edition, buffer) as Edition;
};

export const decodeMasterEdition = (buffer: Buffer) => {
  return deserializeBorsh(
    METADATA_SCHEMA,
    MasterEdition,
    buffer,
  ) as MasterEdition;
};

export async function updateMetadata(
  data: Data | undefined,
  newUpdateAuthority: string | undefined,
  mintKey: PublicKey,
  updateAuthority: PublicKey,
  instructions: TransactionInstruction[],
  metadataAccount?: PublicKey,
) {
  const metadataProgramId = programIds().metadata;

  metadataAccount =
    metadataAccount ||
    (
      await PublicKey.findProgramAddress(
        [
          Buffer.from('metadata'),
          metadataProgramId.toBuffer(),
          mintKey.toBuffer(),
        ],
        metadataProgramId,
      )
    )[0];

  const value = new UpdateMetadataArgs({
    data,
    updateAuthority: !newUpdateAuthority ? undefined : newUpdateAuthority,
  });
  const txnData = Buffer.from(serialize(METADATA_SCHEMA, value));
  const keys = [
    {
      pubkey: metadataAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: updateAuthority,
      isSigner: true,
      isWritable: false,
    },
  ];
  instructions.push(
    new TransactionInstruction({
      keys,
      programId: metadataProgramId,
      data: txnData,
    }),
  );

  return metadataAccount;
}

export async function createMetadata(
  data: Data,
  updateAuthority: PublicKey,
  mintKey: PublicKey,
  mintAuthorityKey: PublicKey,
  instructions: TransactionInstruction[],
  payer: PublicKey,
) {
  const metadataProgramId = programIds().metadata;

  const metadataAccount = (
    await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        metadataProgramId.toBuffer(),
        mintKey.toBuffer(),
      ],
      metadataProgramId,
    )
  )[0];

  const value = new CreateMetadataArgs({ data });
  const txnData = Buffer.from(serialize(METADATA_SCHEMA, value));

  const keys = [
    {
      pubkey: metadataAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: mintKey,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: mintAuthorityKey,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: payer,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: updateAuthority,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SystemProgram.programId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SYSVAR_RENT_PUBKEY,
      isSigner: false,
      isWritable: false,
    },
  ];
  instructions.push(
    new TransactionInstruction({
      keys,
      programId: metadataProgramId,
      data: txnData,
    }),
  );

  return metadataAccount;
}

export async function createMasterEdition(
  maxSupply: BN | undefined,
  mintKey: PublicKey,
  masterMintKey: PublicKey,
  updateAuthorityKey: PublicKey,
  mintAuthorityKey: PublicKey,
  instructions: TransactionInstruction[],
  payer: PublicKey,
  authTokenAccount?: PublicKey,
  masterMintAuthority?: PublicKey,
) {
  const metadataProgramId = programIds().metadata;

  const metadataAccount = (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(METADATA_PREFIX),
        metadataProgramId.toBuffer(),
        mintKey.toBuffer(),
      ],
      metadataProgramId,
    )
  )[0];

  const editionAccount = (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(METADATA_PREFIX),
        metadataProgramId.toBuffer(),
        mintKey.toBuffer(),
        Buffer.from(EDITION),
      ],
      metadataProgramId,
    )
  )[0];

  const value = new CreateMasterEditionArgs({ maxSupply: maxSupply || null });
  const data = Buffer.from(serialize(METADATA_SCHEMA, value));

  const keys = [
    {
      pubkey: editionAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: mintKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: masterMintKey,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: updateAuthorityKey,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: mintAuthorityKey,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: metadataAccount,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: payer,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: programIds().token,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SystemProgram.programId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SYSVAR_RENT_PUBKEY,
      isSigner: false,
      isWritable: false,
    },
  ];

  if (authTokenAccount) {
    keys.push({
      pubkey: authTokenAccount,
      isSigner: false,
      isWritable: true,
    });
  }

  if (masterMintAuthority) {
    keys.push({
      pubkey: masterMintAuthority,
      isSigner: true,
      isWritable: false,
    });
  }
  instructions.push(
    new TransactionInstruction({
      keys,
      programId: metadataProgramId,
      data,
    }),
  );
}

export async function mintNewEditionFromMasterEditionViaToken(
  newMint: PublicKey,
  tokenMint: PublicKey,
  newMintAuthority: PublicKey,
  masterMint: PublicKey,
  authorizationTokenHoldingAccount: PublicKey,
  burnAuthority: PublicKey,
  updateAuthorityOfMaster: PublicKey,
  instructions: TransactionInstruction[],
  payer: PublicKey,
) {
  const metadataProgramId = programIds().metadata;

  const newMetadataKey = await getMetadata(newMint);
  const masterMetadataKey = await getMetadata(tokenMint);
  const newEdition = await getEdition(newMint);
  const masterEdition = await getEdition(tokenMint);

  const data = Buffer.from([3]);

  const keys = [
    {
      pubkey: newMetadataKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: newEdition,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: masterEdition,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: newMint,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: newMintAuthority,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: masterMint,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: authorizationTokenHoldingAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: burnAuthority,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: payer,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: updateAuthorityOfMaster,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: masterMetadataKey,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: programIds().token,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SystemProgram.programId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SYSVAR_RENT_PUBKEY,
      isSigner: false,
      isWritable: false,
    },
  ];
  instructions.push(
    new TransactionInstruction({
      keys,
      programId: metadataProgramId,
      data,
    }),
  );
}

export async function getEdition(tokenMint: PublicKey): Promise<PublicKey> {
  const PROGRAM_IDS = programIds();

  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(METADATA_PREFIX),
        PROGRAM_IDS.metadata.toBuffer(),
        tokenMint.toBuffer(),
        Buffer.from(EDITION),
      ],
      PROGRAM_IDS.metadata,
    )
  )[0];
}

export async function getMetadata(tokenMint: PublicKey): Promise<PublicKey> {
  const PROGRAM_IDS = programIds();

  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from(METADATA_PREFIX),
        PROGRAM_IDS.metadata.toBuffer(),
        tokenMint.toBuffer(),
      ],
      PROGRAM_IDS.metadata,
    )
  )[0];
}
