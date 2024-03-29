import { Account, Connection, TransactionInstruction } from '@solana/web3.js';
import {
  SequenceType,
  sendTransactions,
  sendTransactionWithRetry,
} from '@oyster/common';

import { WhitelistedCreator } from '../models/metaplex';
import { setStore } from '../models/metaplex/setStore';
import { setWhitelistedCreator } from '../models/metaplex/setWhitelistedCreator';

// TODO if this becomes very slow move to batching txns like we do with settle.ts
// but given how little this should be used keep it simple
export async function saveAdmin(
  connection: Connection,
  wallet: any,
  isPublic: boolean,
  whitelistedCreators: WhitelistedCreator[],
) {
  let signers: Array<Account[]> = [];
  let instructions: Array<TransactionInstruction[]> = [];

  let storeSigners: Account[] = [];
  let storeInstructions: TransactionInstruction[] = [];

  await setStore(
    isPublic,
    wallet.publicKey,
    wallet.publicKey,
    storeInstructions,
  );
  signers.push(storeSigners);
  instructions.push(storeInstructions);

  for (let i = 0; i < whitelistedCreators.length; i++) {
    const wc = whitelistedCreators[i];
    let wcSigners: Account[] = [];
    let wcInstructions: TransactionInstruction[] = [];

    await setWhitelistedCreator(
      wc.address,
      wc.activated,
      wallet.publicKey,
      wallet.publicKey,
      wcInstructions,
    );
    signers.push(wcSigners);
    instructions.push(wcInstructions);
  }

  instructions.length == 1
    ? await sendTransactionWithRetry(
        connection,
        wallet,
        instructions[0],
        signers[0],
        'single',
      )
    : await sendTransactions(
        connection,
        wallet,
        instructions,
        signers,
        SequenceType.StopOnFailure,
        'single',
      );
}
