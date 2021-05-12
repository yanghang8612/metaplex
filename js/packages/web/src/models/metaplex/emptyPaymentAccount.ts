import { programIds } from '@oyster/common';
import {
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import { serialize } from 'borsh';

import { EmptyPaymentAccountArgs, SCHEMA } from '.';

export async function emptyPaymentAccount(
  acceptPayment: PublicKey,
  destination: PublicKey,
  auctionManager: PublicKey,
  auctionManagerAuthority: PublicKey,
  instructions: TransactionInstruction[],
) {
  const PROGRAM_IDS = programIds();

  const value = new EmptyPaymentAccountArgs();
  const data = Buffer.from(serialize(SCHEMA, value));

  const keys = [
    {
      pubkey: acceptPayment,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: destination,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: auctionManagerAuthority,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: auctionManager,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: PROGRAM_IDS.token,
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
      programId: PROGRAM_IDS.metaplex,
      data,
    }),
  );
}
