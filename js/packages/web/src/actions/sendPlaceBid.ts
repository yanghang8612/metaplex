import {
  Account,
  Connection,
  PublicKey,
  TransactionInstruction,
} from '@solana/web3.js';
import {
  actions,
  sendTransactionWithRetry,
  placeBid,
  programIds,
  models,
  cache,
  TokenAccount,
  ensureWrappedAccount,
  toLamports,
  ParsedAccount,
} from '@oyster/common';

import { AccountLayout, MintInfo, Token } from '@solana/spl-token';
import { AuctionView } from '../hooks';
import BN from 'bn.js';
const { createTokenAccount } = actions;
const { approve } = models;

export async function sendPlaceBid(
  connection: Connection,
  wallet: any,
  bidderAccount: PublicKey,
  auctionView: AuctionView,
  // value entered by the user adjust to decimals of the mint
  amount: number,
) {
  const tokenAccount = cache.get(bidderAccount) as TokenAccount;
  const mint = cache.get(tokenAccount.info.mint) as ParsedAccount<MintInfo>;
  let lamports = toLamports(amount, mint.info);

  let signers: Account[] = [];
  let instructions: TransactionInstruction[] = [];
  let cleanupInstructions: TransactionInstruction[] = [];

  const accountRentExempt = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span,
  );

  let bidderPotTokenAccount: PublicKey;
  if (!auctionView.myBidderPot) {
    bidderPotTokenAccount = createTokenAccount(
      instructions,
      wallet.publicKey,
      accountRentExempt,
      auctionView.auction.info.tokenMint,
      auctionView.auction.pubkey,
      signers,
    );
  } else bidderPotTokenAccount = auctionView.myBidderPot?.info.bidderPot;

  const toAccount = ensureWrappedAccount(
    instructions,
    cleanupInstructions,
    tokenAccount,
    wallet.publicKey,
    lamports,
    signers,
  );

  const transferAuthority = approve(
    instructions,
    cleanupInstructions,
    toAccount,
    wallet.publicKey,
    lamports,
  );

  signers.push(transferAuthority);

  const bid = await placeBid(
    toAccount,
    bidderPotTokenAccount,
    auctionView.auction.info.tokenMint,
    transferAuthority.publicKey,
    wallet.publicKey,
    auctionView.auctionManager.info.vault,
    new BN(lamports),
    instructions,
  );

  await sendTransactionWithRetry(
    connection,
    wallet,
    [...instructions, ...cleanupInstructions],
    signers,
    'single',
  );

  return bid;
}
