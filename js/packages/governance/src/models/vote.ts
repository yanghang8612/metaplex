import {
  PublicKey,
  SYSVAR_CLOCK_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import { utils } from '@oyster/common';
import * as Layout from '../utils/layout';

import * as BufferLayout from 'buffer-layout';
import { GovernanceInstruction } from './governance';
import BN from 'bn.js';

/// [Requires Voting tokens]
/// Burns voting tokens, indicating you approve and/or disapprove of running this set of transactions. If you tip the consensus,
/// then the transactions can begin to be run at their time slots when people click execute. You are then given yes and/or no tokens.
///
///   0. `[writable]` Governance voting record account.
///                   Can be uninitialized or initialized(if already used once in this proposal)
///                   Must have address with PDA having seed tuple [Governance acct key, proposal key, your voting account key]
///   1. `[writable]` Proposal state account.
///   2. `[writable]` Your Voting account.
///   3. `[writable]` Your Yes-Voting account.
///   4. `[writable]` Your No-Voting account.
///   5. `[writable]` Voting mint account.
///   6. `[writable]` Yes Voting mint account.
///   7. `[writable]` No Voting mint account.
///   8. `[]` Source mint account
///   9. `[]` Proposal account.
///   10. `[]` Governance account.
///   12. `[]` Transfer authority
///   13. `[]` Governance program mint authority (pda of seed Proposal key)
///   14. `[]` Token program account.
///   15. `[]` Clock sysvar.
export const voteInstruction = (
  governanceVotingRecord: PublicKey,
  proposalStateAccount: PublicKey,
  votingAccount: PublicKey,
  yesVotingAccount: PublicKey,
  noVotingAccount: PublicKey,
  votingMint: PublicKey,
  yesVotingMint: PublicKey,
  noVotingMint: PublicKey,
  sourceMint: PublicKey,
  proposalAccount: PublicKey,
  governance: PublicKey,
  transferAuthority: PublicKey,
  mintAuthority: PublicKey,
  yesVotingTokenAmount: number,
  noVotingTokenAmount: number,
): TransactionInstruction => {
  const PROGRAM_IDS = utils.programIds();

  const dataLayout = BufferLayout.struct([
    BufferLayout.u8('instruction'),
    Layout.uint64('yesVotingTokenAmount'),
    Layout.uint64('noVotingTokenAmount'),
  ]);

  const data = Buffer.alloc(dataLayout.span);

  dataLayout.encode(
    {
      instruction: GovernanceInstruction.Vote,
      yesVotingTokenAmount: new BN(yesVotingTokenAmount),
      noVotingTokenAmount: new BN(noVotingTokenAmount),
    },
    data,
  );

  const keys = [
    { pubkey: governanceVotingRecord, isSigner: false, isWritable: true },
    { pubkey: proposalStateAccount, isSigner: false, isWritable: true },
    { pubkey: votingAccount, isSigner: false, isWritable: true },
    { pubkey: yesVotingAccount, isSigner: false, isWritable: true },
    { pubkey: noVotingAccount, isSigner: false, isWritable: true },
    { pubkey: votingMint, isSigner: false, isWritable: true },
    { pubkey: yesVotingMint, isSigner: false, isWritable: true },
    { pubkey: noVotingMint, isSigner: false, isWritable: true },
    { pubkey: sourceMint, isSigner: false, isWritable: false },
    { pubkey: proposalAccount, isSigner: false, isWritable: false },
    { pubkey: governance, isSigner: false, isWritable: false },
    { pubkey: transferAuthority, isSigner: true, isWritable: false },
    { pubkey: mintAuthority, isSigner: false, isWritable: false },
    { pubkey: PROGRAM_IDS.token, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
  ];

  return new TransactionInstruction({
    keys,
    programId: PROGRAM_IDS.governance.programId,
    data,
  });
};
