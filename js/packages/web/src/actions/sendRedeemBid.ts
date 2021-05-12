import {
  Account,
  Connection,
  PublicKey,
  TransactionInstruction,
} from '@solana/web3.js';
import {
  actions,
  ParsedAccount,
  programIds,
  models,
  TokenAccount,
  createMint,
  mintNewEditionFromMasterEditionViaToken,
  SafetyDepositBox,
  SequenceType,
  sendTransactions,
  cache,
  ensureWrappedAccount,
} from '@oyster/common';

import { AccountLayout, MintLayout, Token } from '@solana/spl-token';
import { AuctionView, AuctionViewItem } from '../hooks';
import {
  EditionType,
  NonWinningConstraint,
  redeemBid,
  redeemMasterEditionBid,
  redeemOpenEditionBid,
  WinningConfig,
  WinningConstraint,
} from '../models/metaplex';
import { claimBid } from '../models/metaplex/claimBid';
import { setupCancelBid } from './cancelBid';
const { createTokenAccount } = actions;
const { approve } = models;

export function eligibleForOpenEditionGivenWinningIndex(
  winnerIndex: number | null,
  auctionView: AuctionView,
) {
  return (
    (winnerIndex === null &&
      auctionView.auctionManager.info.settings
        .openEditionNonWinningConstraint !=
        NonWinningConstraint.NoOpenEdition) ||
    (winnerIndex !== null &&
      auctionView.auctionManager.info.settings.openEditionWinnerConstraint !=
        WinningConstraint.NoOpenEdition)
  );
}

export async function sendRedeemBid(
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

  const mintRentExempt = await connection.getMinimumBalanceForRentExemption(
    MintLayout.span,
  );

  let winnerIndex = null;
  if (auctionView.myBidderPot?.pubkey)
    winnerIndex = auctionView.auction.info.bidState.getWinnerIndex(
      auctionView.myBidderPot?.pubkey,
    );
  console.log('Winner index', winnerIndex);

  if (winnerIndex !== null) {
    const winningConfig =
      auctionView.auctionManager.info.settings.winningConfigs[winnerIndex];
    const item = auctionView.items[winnerIndex];
    const safetyDeposit = item.safetyDeposit;
    switch (winningConfig.editionType) {
      case EditionType.LimitedEdition:
        console.log('Redeeming limited');
        await setupRedeemLimitedInstructions(
          auctionView,
          accountsByMint,
          accountRentExempt,
          mintRentExempt,
          wallet,
          safetyDeposit,
          item,
          signers,
          instructions,
          winningConfig,
        );
        break;
      case EditionType.MasterEdition:
        console.log('Redeeming master');
        await setupRedeemMasterInstructions(
          auctionView,
          accountsByMint,
          accountRentExempt,
          wallet,
          safetyDeposit,
          item,
          signers,
          instructions,
        );
        break;
      case EditionType.NA:
        console.log('Redeeming normal');
        await setupRedeemInstructions(
          auctionView,
          accountsByMint,
          accountRentExempt,
          wallet,
          safetyDeposit,
          signers,
          instructions,
        );
        break;
    }

    if (auctionView.myBidderMetadata && auctionView.myBidderPot) {
      let claimSigners: Account[] = [];
      let claimInstructions: TransactionInstruction[] = [];
      instructions.push(claimInstructions);
      signers.push(claimSigners);
      console.log('Claimed');
      await claimBid(
        auctionView.auctionManager.info.acceptPayment,
        auctionView.myBidderMetadata.info.bidderPubkey,
        auctionView.myBidderPot?.info.bidderPot,
        auctionView.vault.pubkey,
        auctionView.auction.info.tokenMint,
        claimInstructions,
      );
    }
  } else {
    // If you didnt win, you must have a bid we can refund before we check for open editions.
    await setupCancelBid(
      auctionView,
      accountsByMint,
      accountRentExempt,
      wallet,
      signers,
      instructions,
    );
  }

  if (
    auctionView.openEditionItem &&
    eligibleForOpenEditionGivenWinningIndex(winnerIndex, auctionView)
  ) {
    const item = auctionView.openEditionItem;
    const safetyDeposit = item.safetyDeposit;
    await setupRedeemOpenInstructions(
      auctionView,
      accountsByMint,
      accountRentExempt,
      mintRentExempt,
      wallet,
      safetyDeposit,
      item,
      signers,
      instructions,
    );
  }

  await sendTransactions(
    connection,
    wallet,
    instructions,
    signers,
    SequenceType.StopOnFailure,
    'single',
  );
}

async function setupRedeemInstructions(
  auctionView: AuctionView,
  accountsByMint: Map<string, TokenAccount>,
  accountRentExempt: number,
  wallet: any,
  safetyDeposit: ParsedAccount<SafetyDepositBox>,
  signers: Array<Account[]>,
  instructions: Array<TransactionInstruction[]>,
) {
  let winningPrizeSigner: Account[] = [];
  let winningPrizeInstructions: TransactionInstruction[] = [];

  signers.push(winningPrizeSigner);
  instructions.push(winningPrizeInstructions);
  if (auctionView.myBidderMetadata) {
    let newTokenAccount = accountsByMint.get(
      safetyDeposit.info.tokenMint.toBase58(),
    )?.pubkey;
    if (!newTokenAccount)
      newTokenAccount = createTokenAccount(
        winningPrizeInstructions,
        wallet.publicKey,
        accountRentExempt,
        safetyDeposit.info.tokenMint,
        wallet.publicKey,
        winningPrizeSigner,
      );

    await redeemBid(
      auctionView.auctionManager.info.vault,
      safetyDeposit.info.store,
      newTokenAccount,
      safetyDeposit.pubkey,
      auctionView.vault.info.fractionMint,
      auctionView.myBidderMetadata.info.bidderPubkey,
      wallet.publicKey,
      winningPrizeInstructions,
    );
  }
}

async function setupRedeemMasterInstructions(
  auctionView: AuctionView,
  accountsByMint: Map<string, TokenAccount>,
  accountRentExempt: number,
  wallet: any,
  safetyDeposit: ParsedAccount<SafetyDepositBox>,
  item: AuctionViewItem,
  signers: Array<Account[]>,
  instructions: Array<TransactionInstruction[]>,
) {
  let winningPrizeSigner: Account[] = [];
  let winningPrizeInstructions: TransactionInstruction[] = [];

  signers.push(winningPrizeSigner);
  instructions.push(winningPrizeInstructions);
  if (auctionView.myBidderMetadata) {
    let newTokenAccount = accountsByMint.get(
      safetyDeposit.info.tokenMint.toBase58(),
    )?.pubkey;
    if (!newTokenAccount)
      newTokenAccount = createTokenAccount(
        winningPrizeInstructions,
        wallet.publicKey,
        accountRentExempt,
        safetyDeposit.info.tokenMint,
        wallet.publicKey,
        winningPrizeSigner,
      );

    await redeemMasterEditionBid(
      auctionView.auctionManager.info.vault,
      safetyDeposit.info.store,
      newTokenAccount,
      safetyDeposit.pubkey,
      auctionView.vault.info.fractionMint,
      auctionView.myBidderMetadata.info.bidderPubkey,
      wallet.publicKey,
      winningPrizeInstructions,
      item.metadata.pubkey,
      wallet.publicKey,
    );
  }
}

async function setupRedeemLimitedInstructions(
  auctionView: AuctionView,
  accountsByMint: Map<string, TokenAccount>,
  accountRentExempt: number,
  mintRentExempt: number,
  wallet: any,
  safetyDeposit: ParsedAccount<SafetyDepositBox>,
  item: AuctionViewItem,
  signers: Array<Account[]>,
  instructions: Array<TransactionInstruction[]>,
  winningConfig: WinningConfig,
) {
  const updateAuth = item.metadata.info.updateAuthority;

  if (item.masterEdition && updateAuth && auctionView.myBidderMetadata) {
    let newTokenAccount: PublicKey | undefined = accountsByMint.get(
      item.masterEdition.info.masterMint.toBase58(),
    )?.pubkey;

    if (!auctionView.myBidRedemption?.info.bidRedeemed) {
      let winningPrizeSigner: Account[] = [];
      let winningPrizeInstructions: TransactionInstruction[] = [];

      signers.push(winningPrizeSigner);
      instructions.push(winningPrizeInstructions);
      if (!newTokenAccount)
        // TODO: switch to ATA
        newTokenAccount = createTokenAccount(
          winningPrizeInstructions,
          wallet.publicKey,
          accountRentExempt,
          item.masterEdition.info.masterMint,
          wallet.publicKey,
          winningPrizeSigner,
        );

      await redeemBid(
        auctionView.auctionManager.info.vault,
        safetyDeposit.info.store,
        newTokenAccount,
        safetyDeposit.pubkey,
        auctionView.vault.info.fractionMint,
        auctionView.myBidderMetadata.info.bidderPubkey,
        wallet.publicKey,
        winningPrizeInstructions,
      );

      for (let i = 0; i < winningConfig.amount; i++) {
        let cashInLimitedPrizeAuthorizationTokenSigner: Account[] = [];
        let cashInLimitedPrizeAuthorizationTokenInstruction: TransactionInstruction[] =
          [];
        signers.push(cashInLimitedPrizeAuthorizationTokenSigner);
        instructions.push(cashInLimitedPrizeAuthorizationTokenInstruction);

        const newLimitedEditionMint = createMint(
          cashInLimitedPrizeAuthorizationTokenInstruction,
          wallet.publicKey,
          mintRentExempt,
          0,
          wallet.publicKey,
          wallet.publicKey,
          cashInLimitedPrizeAuthorizationTokenSigner,
        );
        const newLimitedEdition = createTokenAccount(
          cashInLimitedPrizeAuthorizationTokenInstruction,
          wallet.publicKey,
          accountRentExempt,
          newLimitedEditionMint,
          wallet.publicKey,
          cashInLimitedPrizeAuthorizationTokenSigner,
        );

        cashInLimitedPrizeAuthorizationTokenInstruction.push(
          Token.createMintToInstruction(
            programIds().token,
            newLimitedEditionMint,
            newLimitedEdition,
            wallet.publicKey,
            [],
            1,
          ),
        );

        const burnAuthority = approve(
          cashInLimitedPrizeAuthorizationTokenInstruction,
          [],
          newTokenAccount,
          wallet.publicKey,
          1,
        );

        cashInLimitedPrizeAuthorizationTokenSigner.push(burnAuthority);

        mintNewEditionFromMasterEditionViaToken(
          newLimitedEditionMint,
          item.metadata.info.mint,
          wallet.publicKey,
          item.masterEdition.info.masterMint,
          newTokenAccount,
          burnAuthority.publicKey,
          updateAuth,
          cashInLimitedPrizeAuthorizationTokenInstruction,
          wallet.publicKey,
        );
      }
    }
  }
}

async function setupRedeemOpenInstructions(
  auctionView: AuctionView,
  accountsByMint: Map<string, TokenAccount>,
  accountRentExempt: number,
  mintRentExempt: number,
  wallet: any,
  safetyDeposit: ParsedAccount<SafetyDepositBox>,
  item: AuctionViewItem,
  signers: Array<Account[]>,
  instructions: Array<TransactionInstruction[]>,
) {
  const updateAuth = item.metadata.info.updateAuthority;
  let tokenAccount = accountsByMint.get(
    auctionView.auction.info.tokenMint.toBase58(),
  );
  const mint = cache.get(auctionView.auction.info.tokenMint);

  if (
    item.masterEdition &&
    updateAuth &&
    auctionView.myBidderMetadata &&
    tokenAccount &&
    mint
  ) {
    let newTokenAccount: PublicKey | undefined = accountsByMint.get(
      item.masterEdition.info.masterMint.toBase58(),
    )?.pubkey;

    if (!auctionView.myBidRedemption?.info.bidRedeemed) {
      let winningPrizeSigner: Account[] = [];
      let winningPrizeInstructions: TransactionInstruction[] = [];
      let cleanupInstructions: TransactionInstruction[] = [];

      signers.push(winningPrizeSigner);
      if (!newTokenAccount)
        newTokenAccount = createTokenAccount(
          winningPrizeInstructions,
          wallet.publicKey,
          accountRentExempt,
          item.masterEdition.info.masterMint,
          wallet.publicKey,
          winningPrizeSigner,
        );

      let price: number = auctionView.auctionManager.info.settings
        .openEditionFixedPrice
        ? auctionView.auctionManager.info.settings.openEditionFixedPrice.toNumber()
        : auctionView.myBidderMetadata.info.lastBid.toNumber() || 0;

      const payingSolAccount = ensureWrappedAccount(
        winningPrizeInstructions,
        cleanupInstructions,
        tokenAccount,
        wallet.publicKey,
        price + accountRentExempt,
        winningPrizeSigner,
      );

      const transferAuthority = approve(
        winningPrizeInstructions,
        cleanupInstructions,
        payingSolAccount,
        wallet.publicKey,
        price,
      );

      winningPrizeSigner.push(transferAuthority);

      await redeemOpenEditionBid(
        auctionView.auctionManager.info.vault,
        safetyDeposit.info.store,
        newTokenAccount,
        safetyDeposit.pubkey,
        auctionView.vault.info.fractionMint,
        auctionView.myBidderMetadata.info.bidderPubkey,
        wallet.publicKey,
        winningPrizeInstructions,
        item.metadata.info.mint,
        item.masterEdition.info.masterMint,
        transferAuthority.publicKey,
        auctionView.auctionManager.info.acceptPayment,
        payingSolAccount,
      );

      instructions.push([...winningPrizeInstructions, ...cleanupInstructions]);
    }

    if (newTokenAccount) {
      let cashInOpenPrizeAuthorizationTokenSigner: Account[] = [];
      let cashInOpenPrizeAuthorizationTokenInstruction: TransactionInstruction[] =
        [];
      signers.push(cashInOpenPrizeAuthorizationTokenSigner);
      instructions.push(cashInOpenPrizeAuthorizationTokenInstruction);

      const newOpenEditionMint = createMint(
        cashInOpenPrizeAuthorizationTokenInstruction,
        wallet.publicKey,
        mintRentExempt,
        0,
        wallet.publicKey,
        wallet.publicKey,
        cashInOpenPrizeAuthorizationTokenSigner,
      );
      const newOpenEdition = createTokenAccount(
        cashInOpenPrizeAuthorizationTokenInstruction,
        wallet.publicKey,
        accountRentExempt,
        newOpenEditionMint,
        wallet.publicKey,
        cashInOpenPrizeAuthorizationTokenSigner,
      );

      cashInOpenPrizeAuthorizationTokenInstruction.push(
        Token.createMintToInstruction(
          programIds().token,
          newOpenEditionMint,
          newOpenEdition,
          wallet.publicKey,
          [],
          1,
        ),
      );

      const burnAuthority = approve(
        cashInOpenPrizeAuthorizationTokenInstruction,
        [],
        newTokenAccount,
        wallet.publicKey,
        1,
      );

      cashInOpenPrizeAuthorizationTokenSigner.push(burnAuthority);

      await mintNewEditionFromMasterEditionViaToken(
        newOpenEditionMint,
        item.metadata.info.mint,
        wallet.publicKey,
        item.masterEdition.info.masterMint,
        newTokenAccount,
        burnAuthority.publicKey,
        updateAuth,
        cashInOpenPrizeAuthorizationTokenInstruction,
        wallet.publicKey,
      );
    }
  }
}
