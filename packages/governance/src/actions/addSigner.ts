import {
  Account,
  Connection,
  PublicKey,
  TransactionInstruction,
} from '@solana/web3.js';
import {
  contexts,
  utils,
  models,
  ParsedAccount,
  actions,
} from '@oyster/common';

import {
  GOVERNANCE_AUTHORITY_SEED,
  Proposal,
  ProposalState,
} from '../models/governance';
import { AccountLayout } from '@solana/spl-token';
import { addSignerInstruction } from '../models/addSigner';
const { createTokenAccount } = actions;
const { sendTransaction } = contexts.Connection;
const { notify } = utils;
const { approve } = models;

export const addSigner = async (
  connection: Connection,
  wallet: any,
  proposal: ParsedAccount<Proposal>,
  state: ParsedAccount<ProposalState>,
  adminAccount: PublicKey,
  newSignatoryAccountOwner: PublicKey,
) => {
  const PROGRAM_IDS = utils.programIds();

  let signers: Account[] = [];
  let instructions: TransactionInstruction[] = [];

  const accountRentExempt = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span,
  );

  const newSignerAccount = createTokenAccount(
    instructions,
    wallet.publicKey,
    accountRentExempt,
    proposal.info.signatoryMint,
    newSignatoryAccountOwner,
    signers,
  );

  const [mintAuthority] = await PublicKey.findProgramAddress(
    [Buffer.from(GOVERNANCE_AUTHORITY_SEED), proposal.pubkey.toBuffer()],
    PROGRAM_IDS.governance.programId,
  );

  const transferAuthority = approve(
    instructions,
    [],
    adminAccount,
    wallet.publicKey,
    1,
  );
  signers.push(transferAuthority);

  instructions.push(
    addSignerInstruction(
      newSignerAccount,
      proposal.info.signatoryMint,
      adminAccount,
      proposal.info.adminValidation,
      state.pubkey,
      proposal.pubkey,
      transferAuthority.publicKey,
      mintAuthority,
    ),
  );

  notify({
    message: 'Adding signer...',
    description: 'Please wait...',
    type: 'warn',
  });

  try {
    let tx = await sendTransaction(
      connection,
      wallet,
      instructions,
      signers,
      true,
    );

    notify({
      message: 'Signer added.',
      type: 'success',
      description: `Transaction - ${tx}`,
    });
  } catch (ex) {
    console.error(ex);
    throw new Error();
  }
};
