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
  bidderTokenAccount: PublicKey,
  auctionView: AuctionView,
  // value entered by the user adjust to decimals of the mint
  amount: number,
) {
  const tokenAccount = cache.get(bidderTokenAccount) as TokenAccount;
  const mint = cache.get(tokenAccount.info.mint) as ParsedAccount<MintInfo>;

  let signers: Account[] = [];
  let instructions: TransactionInstruction[] = [];
  let cleanupInstructions: TransactionInstruction[] = [];

  const accountRentExempt = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span,
  );

  let lamports = toLamports(amount, mint.info) + accountRentExempt;

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

  const payingSolAccount = ensureWrappedAccount(
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
    payingSolAccount,
    wallet.publicKey,
    lamports - accountRentExempt,
  );

  signers.push(transferAuthority);

  await placeBid(
    wallet.publicKey,
    payingSolAccount,
    bidderPotTokenAccount,
    auctionView.auction.info.tokenMint,
    transferAuthority.publicKey,
    wallet.publicKey,
    auctionView.auctionManager.info.vault,
    new BN(lamports - accountRentExempt),
    instructions,
  );

  await sendTransactionWithRetry(
    connection,
    wallet,
    [...instructions, ...cleanupInstructions],
    signers,
    'single',
  );
}
