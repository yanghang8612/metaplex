use {
    crate::{PROGRAM_PUBKEY, TOKEN_PROGRAM_PUBKEY, VAULT_PROGRAM_PUBKEY},
    arrayref::array_ref,
    clap::ArgMatches,
    solana_clap_utils::input_parsers::pubkey_of,
    solana_client::rpc_client::RpcClient,
    solana_program::{
        borsh::try_from_slice_unchecked, instruction::Instruction, program_pack::Pack,
        system_instruction::create_account,
    },
    solana_sdk::{
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
    },
    spl_auction::processor::{AuctionData, BidderMetadata},
    spl_metaplex::{
        instruction::{
            create_redeem_bid_instruction, create_redeem_limited_edition_bid_instruction,
            create_redeem_master_edition_bid_instruction,
            create_redeem_open_edition_bid_instruction,
        },
        state::{AuctionManager, OriginalAuthorityLookup, WinningConfig},
    },
    spl_token::{
        instruction::{approve, initialize_account, initialize_mint, mint_to},
        state::{Account, Mint},
    },
    spl_token_metadata::state::{Metadata, EDITION},
    spl_token_vault::state::{SafetyDepositBox, Vault, SAFETY_DEPOSIT_KEY},
    std::{collections::HashMap, str::FromStr},
};

struct BaseAccountList {
    auction_manager: Pubkey,
    store: Pubkey,
    destination: Pubkey,
    bid_redemption: Pubkey,
    safety_deposit_box: Pubkey,
    fraction_mint: Pubkey,
    vault: Pubkey,
    auction: Pubkey,
    bidder_metadata: Pubkey,
    bidder: Pubkey,
    payer: Pubkey,
    token_vault_program: Pubkey,
}

fn redeem_bid_na_type<'a>(
    base_account_list: BaseAccountList,
    winning_config: WinningConfig,
    safety_deposit: &SafetyDepositBox,
    program_id: &Pubkey,
    token_program: &Pubkey,
    instructions: &'a mut Vec<Instruction>,
    client: &RpcClient,
) -> Vec<Instruction> {
    println!("You are redeeming a normal token.");

    let BaseAccountList {
        auction_manager,
        store,
        destination,
        bid_redemption,
        safety_deposit_box,
        fraction_mint,
        vault,
        auction,
        bidder_metadata,
        bidder,
        payer,
        token_vault_program,
    } = base_account_list;
    let transfer_seeds = [
        spl_token_vault::state::PREFIX.as_bytes(),
        token_vault_program.as_ref(),
    ];
    let (transfer_authority, _) =
        Pubkey::find_program_address(&transfer_seeds, &token_vault_program);

    instructions.push(create_account(
        &payer,
        &destination,
        client
            .get_minimum_balance_for_rent_exemption(Account::LEN)
            .unwrap(),
        Account::LEN as u64,
        &token_program,
    ));
    instructions.push(
        initialize_account(
            &token_program,
            &destination,
            &safety_deposit.token_mint,
            &bidder,
        )
        .unwrap(),
    );
    instructions.push(
        approve(
            token_program,
            &base_account_list.destination,
            &transfer_authority,
            &base_account_list.bidder,
            &[&base_account_list.bidder],
            winning_config.amount.into(),
        )
        .unwrap(),
    );

    instructions.push(create_redeem_bid_instruction(
        *program_id,
        auction_manager,
        store,
        destination,
        bid_redemption,
        safety_deposit_box,
        fraction_mint,
        vault,
        auction,
        bidder_metadata,
        bidder,
        payer,
        token_vault_program,
        transfer_authority,
    ));

    let mut new_instructions: Vec<Instruction> = vec![];
    for n in 0..instructions.len() {
        new_instructions.push(instructions[n].clone());
    }
    new_instructions
}

fn redeem_bid_limited_edition_type<'a>(
    base_account_list: BaseAccountList,
    new_mint: &'a Keypair,
    safety_deposit: &SafetyDepositBox,
    program_id: &Pubkey,
    instructions: &'a mut Vec<Instruction>,
    token_metadata_key: &Pubkey,
    client: &RpcClient,
) -> Vec<Instruction> {
    println!("You are redeeming a limited edition.");

    let BaseAccountList {
        auction_manager,
        store,
        destination,
        bid_redemption,
        safety_deposit_box,
        fraction_mint,
        vault,
        auction,
        bidder_metadata,
        bidder,
        payer,
        token_vault_program,
    } = base_account_list;

    let master_metadata_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        &safety_deposit.token_mint.as_ref(),
    ];
    let (master_metadata_key, _) =
        Pubkey::find_program_address(master_metadata_seeds, &token_metadata_key);

    let master_metadata_account = client.get_account(&master_metadata_key).unwrap();
    let master_metadata: Metadata =
        try_from_slice_unchecked(&master_metadata_account.data).unwrap();

    let master_name_symbol_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        master_metadata.data.name.as_bytes(),
        master_metadata.data.symbol.as_bytes(),
    ];
    let (master_name_symbol_key, _) =
        Pubkey::find_program_address(master_name_symbol_seeds, &token_metadata_key);

    let master_edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        safety_deposit.token_mint.as_ref(),
        EDITION.as_bytes(),
    ];
    let (master_edition_key, _) =
        Pubkey::find_program_address(master_edition_seeds, &token_metadata_key);

    let new_mint_key = new_mint.pubkey();

    let new_metadata_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        &new_mint_key.as_ref(),
    ];
    let (new_metadata_key, _) =
        Pubkey::find_program_address(new_metadata_seeds, &token_metadata_key);

    let new_edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        new_mint_key.as_ref(),
        EDITION.as_bytes(),
    ];
    let (new_edition_key, _) = Pubkey::find_program_address(new_edition_seeds, &token_metadata_key);

    let original_lookup_authority_seeds = &[
        spl_metaplex::state::PREFIX.as_bytes(),
        &auction.as_ref(),
        &master_metadata_key.as_ref(),
    ];
    let (original_lookup_authority_key, _) =
        Pubkey::find_program_address(original_lookup_authority_seeds, &program_id);

    let original_lookup_account = client.get_account(&original_lookup_authority_key).unwrap();
    let original_lookup: OriginalAuthorityLookup =
        try_from_slice_unchecked(&original_lookup_account.data).unwrap();

    instructions.push(create_redeem_limited_edition_bid_instruction(
        *program_id,
        auction_manager,
        store,
        destination,
        bid_redemption,
        safety_deposit_box,
        fraction_mint,
        vault,
        auction,
        bidder_metadata,
        bidder,
        payer,
        token_vault_program,
        new_metadata_key,
        new_mint.pubkey(),
        payer,
        master_metadata_key,
        master_name_symbol_key,
        new_edition_key,
        master_edition_key,
        original_lookup.original_authority,
        original_lookup_authority_key,
    ));

    let mut new_instructions: Vec<Instruction> = vec![];
    for n in 0..instructions.len() {
        new_instructions.push(instructions[n].clone());
    }
    new_instructions
}

fn create_ancillary_accounts(
    base_account_list: &BaseAccountList,
    new_mint: &Keypair,
    destination_keypair: &Keypair,
    token_program: &Pubkey,
    client: &RpcClient,
    payer_keypair: &Keypair,
) {
    let BaseAccountList {
        auction_manager: _a,
        store: _s,
        destination,
        bid_redemption: _b,
        safety_deposit_box: _sa,
        fraction_mint: _f,
        vault: _v,
        auction: _au,
        bidder_metadata: _bi,
        bidder: _bid,
        payer,
        token_vault_program: _to,
    } = base_account_list;

    // Had to split these out and make them a separate txn because of txn size limits..
    let ancillary_signers = vec![payer_keypair, new_mint, destination_keypair];
    let create_ancillary_account_instructions = [
        create_account(
            &payer,
            &new_mint.pubkey(),
            client
                .get_minimum_balance_for_rent_exemption(Mint::LEN)
                .unwrap(),
            Mint::LEN as u64,
            &token_program,
        ),
        initialize_mint(&token_program, &new_mint.pubkey(), &payer, Some(&payer), 0).unwrap(),
        create_account(
            &payer,
            &destination,
            client
                .get_minimum_balance_for_rent_exemption(Account::LEN)
                .unwrap(),
            Account::LEN as u64,
            &token_program,
        ),
        initialize_account(&token_program, &destination, &new_mint.pubkey(), &payer).unwrap(),
        mint_to(
            &token_program,
            &new_mint.pubkey(),
            &destination,
            &payer,
            &[&payer],
            1,
        )
        .unwrap(),
    ];

    let mut transaction =
        Transaction::new_with_payer(&create_ancillary_account_instructions, Some(&payer));
    let recent_blockhash = client.get_recent_blockhash().unwrap().0;

    transaction.sign(&ancillary_signers, recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();
}

fn redeem_bid_open_edition_type<'a>(
    base_account_list: BaseAccountList,
    manager: &AuctionManager,
    new_mint: &'a Keypair,
    safety_deposit: &SafetyDepositBox,
    program_id: &Pubkey,
    token_program: &Pubkey,
    instructions: &'a mut Vec<Instruction>,
    token_metadata_key: &Pubkey,
    transfer_authority: &Keypair,
) -> Vec<Instruction> {
    println!("You are redeeming an open edition.");

    let BaseAccountList {
        auction_manager,
        store,
        destination,
        bid_redemption,
        safety_deposit_box,
        fraction_mint,
        vault,
        auction,
        bidder_metadata,
        bidder,
        payer,
        token_vault_program,
    } = base_account_list;

    let master_metadata_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        &safety_deposit.token_mint.as_ref(),
    ];
    let (master_metadata_key, _) =
        Pubkey::find_program_address(master_metadata_seeds, &token_metadata_key);

    let master_edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        safety_deposit.token_mint.as_ref(),
        EDITION.as_bytes(),
    ];
    let (master_edition_key, _) =
        Pubkey::find_program_address(master_edition_seeds, &token_metadata_key);

    let new_mint_key = new_mint.pubkey();

    let new_metadata_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        &new_mint_key.as_ref(),
    ];
    let (new_metadata_key, _) =
        Pubkey::find_program_address(new_metadata_seeds, &token_metadata_key);

    let new_edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        new_mint_key.as_ref(),
        EDITION.as_bytes(),
    ];
    let (new_edition_key, _) = Pubkey::find_program_address(new_edition_seeds, &token_metadata_key);

    if let Some(price) = manager.settings.open_edition_fixed_price {
        instructions.push(
            approve(
                token_program,
                &base_account_list.bidder,
                &transfer_authority.pubkey(),
                &base_account_list.bidder,
                &[&base_account_list.bidder],
                price,
            )
            .unwrap(),
        );
    }

    instructions.push(create_redeem_open_edition_bid_instruction(
        *program_id,
        auction_manager,
        store,
        destination,
        bid_redemption,
        safety_deposit_box,
        fraction_mint,
        vault,
        auction,
        bidder_metadata,
        bidder,
        payer,
        token_vault_program,
        new_metadata_key,
        new_mint.pubkey(),
        payer,
        master_metadata_key,
        new_edition_key,
        master_edition_key,
        transfer_authority.pubkey(),
    ));

    let mut new_instructions: Vec<Instruction> = vec![];
    for n in 0..instructions.len() {
        new_instructions.push(instructions[n].clone());
    }

    new_instructions
}

fn redeem_bid_master_edition_type<'a>(
    base_account_list: BaseAccountList,
    safety_deposit: &SafetyDepositBox,
    program_id: &Pubkey,
    token_program: &Pubkey,
    instructions: &'a mut Vec<Instruction>,
    token_metadata_key: &Pubkey,
    client: &RpcClient,
) -> Vec<Instruction> {
    println!("You are redeeming a master edition.");
    let BaseAccountList {
        auction_manager,
        store,
        destination,
        bid_redemption,
        safety_deposit_box,
        fraction_mint,
        vault,
        auction,
        bidder_metadata,
        bidder,
        payer,
        token_vault_program,
    } = base_account_list;

    let master_metadata_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        &safety_deposit.token_mint.as_ref(),
    ];
    let (master_metadata_key, _) =
        Pubkey::find_program_address(master_metadata_seeds, &token_metadata_key);

    let master_metadata_account = client.get_account(&master_metadata_key).unwrap();
    let master_metadata: Metadata =
        try_from_slice_unchecked(&master_metadata_account.data).unwrap();

    let master_name_symbol_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        &token_metadata_key.as_ref(),
        master_metadata.data.name.as_bytes(),
        master_metadata.data.symbol.as_bytes(),
    ];
    let (master_name_symbol_key, _) =
        Pubkey::find_program_address(master_name_symbol_seeds, &token_metadata_key);

    let transfer_seeds = [
        spl_token_vault::state::PREFIX.as_bytes(),
        token_vault_program.as_ref(),
    ];
    let (transfer_authority, _) =
        Pubkey::find_program_address(&transfer_seeds, &token_vault_program);

    instructions.push(create_account(
        &payer,
        &destination,
        client
            .get_minimum_balance_for_rent_exemption(Account::LEN)
            .unwrap(),
        Account::LEN as u64,
        &token_program,
    ));
    instructions.push(
        initialize_account(
            &token_program,
            &destination,
            &safety_deposit.token_mint,
            &bidder,
        )
        .unwrap(),
    );
    instructions.push(
        approve(
            token_program,
            &base_account_list.destination,
            &transfer_authority,
            &base_account_list.bidder,
            &[&base_account_list.bidder],
            1,
        )
        .unwrap(),
    );

    instructions.push(create_redeem_master_edition_bid_instruction(
        *program_id,
        auction_manager,
        store,
        destination,
        bid_redemption,
        safety_deposit_box,
        fraction_mint,
        vault,
        auction,
        bidder_metadata,
        bidder,
        payer,
        token_vault_program,
        master_metadata_key,
        master_name_symbol_key,
        bidder,
        transfer_authority,
    ));

    let mut new_instructions: Vec<Instruction> = vec![];
    for n in 0..instructions.len() {
        new_instructions.push(instructions[n].clone());
    }
    new_instructions
}

pub fn redeem_bid_wrapper(app_matches: &ArgMatches, payer: Keypair, client: RpcClient) {
    let program_key = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();
    let token_key = Pubkey::from_str(TOKEN_PROGRAM_PUBKEY).unwrap();
    let token_metadata_key = spl_token_metadata::id();

    let token_vault_program = Pubkey::from_str(VAULT_PROGRAM_PUBKEY).unwrap();

    let wallet = read_keypair_file(
        app_matches
            .value_of("wallet")
            .unwrap_or_else(|| app_matches.value_of("keypair").unwrap()),
    )
    .unwrap();

    let auction_manager_key = pubkey_of(app_matches, "auction_manager").unwrap();

    let account = client.get_account(&auction_manager_key).unwrap();
    let manager: AuctionManager = try_from_slice_unchecked(&account.data).unwrap();
    let all_vault_accounts = client.get_program_accounts(&token_vault_program).unwrap();

    let mut safety_deposits = HashMap::new();

    for n in 0..all_vault_accounts.len() {
        let obj = &all_vault_accounts[n].1;
        let obj_key = &all_vault_accounts[n].0;
        let type_of_obj = obj.data[0];

        if type_of_obj == SAFETY_DEPOSIT_KEY {
            let pubkey_arr = array_ref![obj.data, 1, 32];
            let pubkey = Pubkey::new_from_array(*pubkey_arr);
            if pubkey == manager.vault {
                let safety_deposit: SafetyDepositBox = try_from_slice_unchecked(&obj.data).unwrap();
                safety_deposits.insert(safety_deposit.order, (safety_deposit, obj_key));
            }
        }
    }
    let wallet_key = wallet.pubkey();
    let meta_path = [
        spl_auction::PREFIX.as_bytes(),
        manager.auction_program.as_ref(),
        manager.auction.as_ref(),
        wallet_key.as_ref(),
        "metadata".as_bytes(),
    ];

    let (meta_key, _) = Pubkey::find_program_address(&meta_path, &manager.auction_program);
    let bidding_metadata = client.get_account(&meta_key).unwrap();
    let auction_data = client.get_account(&manager.auction).unwrap();
    let vault_data = client.get_account(&manager.vault).unwrap();
    let auction: AuctionData = try_from_slice_unchecked(&auction_data.data).unwrap();
    let bid: BidderMetadata = try_from_slice_unchecked(&bidding_metadata.data).unwrap();
    let vault: Vault = try_from_slice_unchecked(&vault_data.data).unwrap();

    let redemption_path = [
        spl_metaplex::state::PREFIX.as_bytes(),
        manager.auction.as_ref(),
        &meta_key.as_ref(),
    ];
    let (bid_redemption_key, _) = Pubkey::find_program_address(&redemption_path, &program_key);

    if let Some(winning_index) = auction.bid_state.is_winner(bid.bidder_pubkey) {
        let destination = Keypair::new();
        let winning_config = manager.settings.winning_configs[winning_index];
        let safety_deposit_result = safety_deposits
            .get(&winning_config.safety_deposit_box_index)
            .unwrap();
        let safety_deposit = &safety_deposit_result.0;
        let safety_deposit_key = safety_deposit_result.1;
        let new_mint = Keypair::new();
        let mut signers: Vec<&Keypair> = vec![&wallet, &payer];
        let mut instructions: Vec<Instruction> = vec![];

        let base_account_list = BaseAccountList {
            auction_manager: auction_manager_key,
            store: safety_deposit.store,
            destination: destination.pubkey(),
            bid_redemption: bid_redemption_key,
            safety_deposit_box: *safety_deposit_key,
            fraction_mint: vault.fraction_mint,
            vault: manager.vault,
            auction: manager.auction,
            bidder_metadata: meta_key,
            bidder: wallet.pubkey(),
            payer: payer.pubkey(),
            token_vault_program,
        };

        let instructions = match winning_config.edition_type {
            spl_metaplex::state::EditionType::NA => {
                signers.push(&destination);
                redeem_bid_na_type(
                    base_account_list,
                    winning_config,
                    safety_deposit,
                    &program_key,
                    &token_key,
                    &mut instructions,
                    &client,
                )
            }
            spl_metaplex::state::EditionType::LimitedEdition => {
                // Destination made in separate txn here. No need to add signers.
                create_ancillary_accounts(
                    &base_account_list,
                    &new_mint,
                    &destination,
                    &token_key,
                    &client,
                    &payer,
                );

                redeem_bid_limited_edition_type(
                    base_account_list,
                    &new_mint,
                    safety_deposit,
                    &program_key,
                    &mut instructions,
                    &token_metadata_key,
                    &client,
                )
            }
            spl_metaplex::state::EditionType::MasterEdition => {
                signers.push(&destination);
                redeem_bid_master_edition_type(
                    base_account_list,
                    safety_deposit,
                    &program_key,
                    &token_key,
                    &mut instructions,
                    &token_metadata_key,
                    &client,
                )
            }
        };

        let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
        let recent_blockhash = client.get_recent_blockhash().unwrap().0;

        transaction.sign(&signers, recent_blockhash);
        client.send_and_confirm_transaction(&transaction).unwrap();

        println!(
            "Sent prize to {:?}, Now let's see if you have an open edition to redeem...",
            destination.pubkey()
        )
    } else {
        println!("You are not a winner, but lets see if you have open editions to redeem...");
    }

    if let Some(open_edition_config) = manager.settings.open_edition_config {
        println!("This auction has an open edition. Submitting!");
        let safety_deposit_result = safety_deposits.get(&open_edition_config).unwrap();
        let destination = Keypair::new();
        let new_mint = Keypair::new();
        let safety_deposit = &safety_deposit_result.0;
        let safety_deposit_key = safety_deposit_result.1;
        let transfer_authority = Keypair::new();
        let signers = vec![&wallet, &transfer_authority, &payer];
        let mut instructions: Vec<Instruction> = vec![];
        let base_account_list = BaseAccountList {
            auction_manager: auction_manager_key,
            store: safety_deposit.store,
            destination: destination.pubkey(),
            bid_redemption: bid_redemption_key,
            safety_deposit_box: *safety_deposit_key,
            fraction_mint: vault.fraction_mint,
            vault: manager.vault,
            auction: manager.auction,
            bidder_metadata: meta_key,
            bidder: wallet.pubkey(),
            payer: payer.pubkey(),
            token_vault_program,
        };

        create_ancillary_accounts(
            &base_account_list,
            &new_mint,
            &destination,
            &token_key,
            &client,
            &payer,
        );

        let instructions = redeem_bid_open_edition_type(
            base_account_list,
            &manager,
            &new_mint,
            safety_deposit,
            &program_key,
            &token_key,
            &mut instructions,
            &token_metadata_key,
            &transfer_authority,
        );

        let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
        let recent_blockhash = client.get_recent_blockhash().unwrap().0;

        transaction.sign(&signers, recent_blockhash);
        client.send_and_confirm_transaction(&transaction).unwrap();

        println!("Open edition sent to {:?}", destination.pubkey());
    }
}
