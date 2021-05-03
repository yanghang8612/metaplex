import { ParsedAccount } from '@oyster/common';
import { Button, Col, Modal, Row } from 'antd';
import React from 'react';
import {
  Governance,
  Proposal,
  ProposalState,
  ProposalStateStatus,
} from '../../models/governance';
import { LABELS } from '../../constants';
import { depositSourceTokensAndVote } from '../../actions/depositSourceTokensAndVote';
import { contexts, hooks } from '@oyster/common';
import { CheckOutlined, CloseOutlined } from '@ant-design/icons';

import './style.less';

const { useWallet } = contexts.Wallet;
const { useConnection } = contexts.Connection;
const { useAccountByMint } = hooks;

const { confirm } = Modal;
export function Vote({
  proposal,
  state,
  governance,
  yeahVote,
}: {
  proposal: ParsedAccount<Proposal>;
  state: ParsedAccount<ProposalState>;
  governance: ParsedAccount<Governance>;
  yeahVote: boolean;
}) {
  const wallet = useWallet();
  const connection = useConnection();

  const voteAccount = useAccountByMint(proposal.info.votingMint);
  const yesVoteAccount = useAccountByMint(proposal.info.yesVotingMint);
  const noVoteAccount = useAccountByMint(proposal.info.noVotingMint);

  const userTokenAccount = useAccountByMint(proposal.info.sourceMint);

  const eligibleToView =
    userTokenAccount &&
    userTokenAccount.info.amount.toNumber() > 0 &&
    state.info.status === ProposalStateStatus.Voting;

  const [btnLabel, title, msg, icon] = yeahVote
    ? [
        LABELS.VOTE_YEAH,
        LABELS.VOTE_YEAH_QUESTION,
        LABELS.VOTE_YEAH_MSG,
        <CheckOutlined />,
      ]
    : [
        LABELS.VOTE_NAY,
        LABELS.VOTE_NAY_QUESTION,
        LABELS.VOTE_NAY_MSG,
        <CloseOutlined />,
      ];

  return eligibleToView ? (
    <Button
      type="primary"
      icon={icon}
      onClick={() =>
        confirm({
          title: title,
          icon: icon,
          content: (
            <Row>
              <Col span={24}>
                <p>{msg}</p>
              </Col>
            </Row>
          ),
          okText: LABELS.CONFIRM,
          cancelText: LABELS.CANCEL,
          onOk: async () => {
            if (userTokenAccount) {
              const voteAmount = userTokenAccount.info.amount.toNumber();

              const yesTokenAmount = yeahVote ? voteAmount : 0;
              const noTokenAmount = !yeahVote ? voteAmount : 0;

              await depositSourceTokensAndVote(
                connection,
                wallet.wallet,
                proposal,
                voteAccount?.pubkey,
                yesVoteAccount?.pubkey,
                noVoteAccount?.pubkey,
                userTokenAccount.pubkey,
                governance,
                state,
                yesTokenAmount,
                noTokenAmount,
              );
            }
          },
        })
      }
    >
      {btnLabel}
    </Button>
  ) : null;
}
