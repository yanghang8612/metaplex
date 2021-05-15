import { programIds } from '@oyster/common';
import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from '@solana/web3.js';
import { serialize } from 'borsh';

import { getAuctionKeys, SCHEMA, ValidateOpenEditionArgs } from '.';

export async function validateOpenEdition(
  vault: PublicKey,
  openEditionMetadata: PublicKey,
  openEditionAuthority: PublicKey,
  openEditionMasterAccount: PublicKey,
  openEditionMint: PublicKey,
  openEditionMasterMint: PublicKey,
  openEditionMasterMintAuthority: PublicKey,
  auctionManagerAuthority: PublicKey,
  store: PublicKey,
  whitelistedCreatorEntry: PublicKey | undefined,
  instructions: TransactionInstruction[],
) {
  const PROGRAM_IDS = programIds();
  const { auctionManagerKey } = await getAuctionKeys(vault);

  const value = new ValidateOpenEditionArgs();

  const data = Buffer.from(serialize(SCHEMA, value));

  const keys = [
    {
      pubkey: auctionManagerKey,
      isSigner: false,
      isWritable: true,
    },

    {
      pubkey: openEditionMetadata,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: openEditionMint,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: openEditionMasterMint,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: openEditionMasterMintAuthority,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: openEditionAuthority,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: auctionManagerAuthority,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: openEditionMasterAccount,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: whitelistedCreatorEntry || SystemProgram.programId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: store,
      isSigner: false,
      isWritable: false,
    },

    {
      pubkey: vault,
      isSigner: false,
      isWritable: false,
    },

    {
      pubkey: PROGRAM_IDS.token,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: PROGRAM_IDS.metadata,
      isSigner: false,
      isWritable: false,
    },
  ];
  instructions.push(
    new TransactionInstruction({
      keys,
      programId: PROGRAM_IDS.metaplex,
      data,
    }),
  );
}
