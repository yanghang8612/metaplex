import {
  TokenAccount,
  cancelBid,
  cache,
  ensureWrappedAccount,
  sendTransactionWithRetry,
} from '@oyster/common';
import { AccountLayout } from '@solana/spl-token';
import { TransactionInstruction, Account, Connection } from '@solana/web3.js';
import { AuctionView } from '../hooks';

export async function sendCancelBid(
  connection: Connection,
  wallet: any,
  auctionView: AuctionView,
  accountsByMint: Map<string, TokenAccount>,
) {
  let signers: Array<Account[]> = [];
  let instructions: Array<TransactionInstruction[]> = [];
  const accountRentExempt = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span,
  );

  await setupCancelBid(
    auctionView,
    accountsByMint,
    accountRentExempt,
    wallet,
    signers,
    instructions,
  );

  await sendTransactionWithRetry(
    connection,
    wallet,
    instructions[0],
    signers[0],
    'single',
  );
}

export async function setupCancelBid(
  auctionView: AuctionView,
  accountsByMint: Map<string, TokenAccount>,
  accountRentExempt: number,
  wallet: any,
  signers: Array<Account[]>,
  instructions: Array<TransactionInstruction[]>,
) {
  let cancelSigners: Account[] = [];
  let cancelInstructions: TransactionInstruction[] = [];
  let cleanupInstructions: TransactionInstruction[] = [];

  let tokenAccount = accountsByMint.get(
    auctionView.auction.info.tokenMint.toBase58(),
  );
  const mint = cache.get(auctionView.auction.info.tokenMint);

  if (mint && auctionView.myBidderPot) {
    const receivingSolAccount = ensureWrappedAccount(
      cancelInstructions,
      cleanupInstructions,
      tokenAccount,
      wallet.publicKey,
      accountRentExempt,
      cancelSigners,
    );

    await cancelBid(
      wallet.publicKey,
      receivingSolAccount,
      auctionView.myBidderPot.info.bidderPot,
      auctionView.auction.info.tokenMint,
      auctionView.vault.pubkey,
      cancelInstructions,
    );
    signers.push(cancelSigners);
    instructions.push([...cancelInstructions, ...cleanupInstructions]);
  }
}
