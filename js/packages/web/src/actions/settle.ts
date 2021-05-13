import { Account, Connection, TransactionInstruction } from '@solana/web3.js';
import {
  actions,
  ParsedAccount,
  models,
  TokenAccount,
  SequenceType,
  sendTransactions,
  sendTransactionWithRetry,
  BidderMetadata,
  BidderPot,
  ensureWrappedAccount,
} from '@oyster/common';

import { AuctionView } from '../hooks';

import { claimBid } from '../models/metaplex/claimBid';
import { emptyPaymentAccount } from '../models/metaplex/emptyPaymentAccount';
import { AccountLayout } from '@solana/spl-token';

const BATCH_SIZE = 10;
const TRANSACTION_SIZE = 7;
export async function settle(
  connection: Connection,
  wallet: any,
  auctionView: AuctionView,
  existingWrappedSolAccount: TokenAccount,
  bids: ParsedAccount<BidderPot>[],
) {
  let signers: Array<Array<Account[]>> = [];
  let instructions: Array<Array<TransactionInstruction[]>> = [];

  let currSignerBatch: Array<Account[]> = [];
  let currInstrBatch: Array<TransactionInstruction[]> = [];

  let claimBidSigners: Account[] = [];
  let claimBidInstructions: TransactionInstruction[] = [];

  // TODO replace all this with payer account so user doesnt need to click approve several times.

  // Overall we have 10 parallel txns, of up to 7 claims in each txn
  // That's what this loop is building.
  for (let i = 0; i < bids.length; i++) {
    const bid = bids[i];

    await claimBid(
      auctionView.auctionManager.info.acceptPayment,
      bid.info.bidderAct,
      bid.info.bidderPot,
      auctionView.vault.pubkey,
      auctionView.auction.info.tokenMint,
      claimBidInstructions,
    );

    if (claimBidInstructions.length == TRANSACTION_SIZE) {
      currSignerBatch.push(claimBidSigners);
      currInstrBatch.push(claimBidInstructions);
      claimBidSigners = [];
      claimBidInstructions = [];
    }

    if (currInstrBatch.length == BATCH_SIZE) {
      signers.push(currSignerBatch);
      instructions.push(currInstrBatch);
      currSignerBatch = [];
      currInstrBatch = [];
    }
  }

  if (
    claimBidInstructions.length < TRANSACTION_SIZE &&
    claimBidInstructions.length > 0
  ) {
    currSignerBatch.push(claimBidSigners);
    currInstrBatch.push(claimBidInstructions);
  }

  if (currInstrBatch.length <= BATCH_SIZE && currInstrBatch.length > 0) {
    // add the last one on
    signers.push(currSignerBatch);
    instructions.push(currInstrBatch);
  }

  for (let i = 0; i < instructions.length; i++) {
    const instructionBatch = instructions[i];
    const signerBatch = signers[i];
    if (instructionBatch.length >= 2)
      // Pump em through!
      await sendTransactions(
        connection,
        wallet,
        instructionBatch,
        signerBatch,
        SequenceType.Parallel,
        'single',
      );
    else
      await sendTransactionWithRetry(
        connection,
        wallet,
        instructionBatch[0],
        signerBatch[0],
        'single',
      );
  }

  const pullMoneySigners: Account[] = [];
  const pullMoneyInstructions: TransactionInstruction[] = [];
  const cleanupInstructions: TransactionInstruction[] = [];

  const accountRentExempt = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span,
  );

  const receivingSolAccount = ensureWrappedAccount(
    pullMoneyInstructions,
    cleanupInstructions,
    existingWrappedSolAccount,
    wallet.publicKey,
    accountRentExempt,
    pullMoneySigners,
  );

  await emptyPaymentAccount(
    auctionView.auctionManager.info.acceptPayment,
    receivingSolAccount,
    auctionView.auctionManager.pubkey,
    wallet.publicKey,
    pullMoneyInstructions,
  );

  await sendTransactionWithRetry(
    connection,
    wallet,
    [...pullMoneyInstructions, ...cleanupInstructions],
    pullMoneySigners,
    'single',
  );
}
