use {
    crate::{
        settings_utils::{parse_settings, JSONAuctionManagerSettings},
        vault_utils::{activate_vault, add_token_to_vault, combine_vault, initialize_vault},
        AUCTION_PROGRAM_PUBKEY, PROGRAM_PUBKEY, TOKEN_PROGRAM_PUBKEY, VAULT_PROGRAM_PUBKEY,
    },
    clap::ArgMatches,
    solana_clap_utils::input_parsers::pubkey_of,
    solana_client::rpc_client::RpcClient,
    solana_program::{
        borsh::try_from_slice_unchecked, instruction::Instruction, program_pack::Pack,
    },
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction::create_account,
        transaction::Transaction,
    },
    spl_auction::{
        instruction::create_auction_instruction,
        processor::{create_auction::CreateAuctionArgs, WinnerLimit},
    },
    spl_metaplex::{instruction::create_init_auction_manager_instruction, state::AuctionManager},
    spl_token::{instruction::initialize_mint, state::Mint},
    spl_token_metadata::state::{Key, MasterEdition, Metadata, NameSymbolTuple, EDITION},
    spl_token_vault::{
        instruction::create_update_external_price_account_instruction,
        state::MAX_EXTERNAL_ACCOUNT_SIZE,
    },
    std::{convert::TryInto, str::FromStr},
};

fn find_or_initialize_external_account<'a>(
    app_matches: &ArgMatches,
    payer: &Keypair,
    vault_program_key: &Pubkey,
    token_key: &Pubkey,
    client: &RpcClient,
    payer_mint_key: &'a Keypair,
    external_keypair: &'a Keypair,
) -> Pubkey {
    let external_key: Pubkey;
    if !app_matches.is_present("external_price_account") {
        let mut instructions: Vec<Instruction> = vec![];
        let mut signers: Vec<&Keypair> = vec![&payer, &external_keypair];
        instructions.push(create_account(
            &payer.pubkey(),
            &payer_mint_key.pubkey(),
            client
                .get_minimum_balance_for_rent_exemption(Mint::LEN)
                .unwrap(),
            Mint::LEN as u64,
            &token_key,
        ));
        instructions.push(
            initialize_mint(
                &token_key,
                &payer_mint_key.pubkey(),
                &payer.pubkey(),
                Some(&payer.pubkey()),
                0,
            )
            .unwrap(),
        );
        instructions.push(create_account(
            &payer.pubkey(),
            &external_keypair.pubkey(),
            client
                .get_minimum_balance_for_rent_exemption(MAX_EXTERNAL_ACCOUNT_SIZE)
                .unwrap(),
            MAX_EXTERNAL_ACCOUNT_SIZE as u64,
            &vault_program_key,
        ));
        instructions.push(create_update_external_price_account_instruction(
            *vault_program_key,
            external_keypair.pubkey(),
            0,
            payer_mint_key.pubkey(),
            true,
        ));

        signers.push(&payer_mint_key);
        signers.push(&external_keypair);

        let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
        let recent_blockhash = client.get_recent_blockhash().unwrap().0;

        transaction.sign(&signers, recent_blockhash);
        client.send_and_confirm_transaction(&transaction).unwrap();
        external_key = external_keypair.pubkey();
    } else {
        external_key = pubkey_of(app_matches, "external_price_account").unwrap();
    }

    external_key
}

fn find_or_initialize_auction(
    app_matches: &ArgMatches,
    vault_key: &Pubkey,
    program_key: &Pubkey,
    auction_program_key: &Pubkey,
    payer_mint_key: &Pubkey,
    payer: &Keypair,
    client: &RpcClient,
) -> Pubkey {
    let auction_key: Pubkey;
    if !app_matches.is_present("auction") {
        let signers: Vec<&Keypair> = vec![&payer];

        let winner_limit = app_matches
            .value_of("winner_limit")
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap();

        let gap_time = app_matches
            .value_of("gap_time")
            .unwrap_or("1200")
            .parse::<u64>()
            .unwrap();

        let end_time = app_matches
            .value_of("end_time")
            .unwrap_or("1200")
            .parse::<u64>()
            .unwrap();

        let auction_path = [
            spl_auction::PREFIX.as_bytes(),
            auction_program_key.as_ref(),
            &vault_key.to_bytes(),
        ];

        // Derive the address we'll store the auction in, and confirm it matches what we expected the
        // user to provide.
        let (actual_auction_key, _) =
            Pubkey::find_program_address(&auction_path, auction_program_key);

        // You'll notice that the authority IS what will become the auction manager ;)
        let authority_seeds = &[
            spl_metaplex::state::PREFIX.as_bytes(),
            &actual_auction_key.as_ref(),
        ];
        let (auction_manager_key, _) = Pubkey::find_program_address(authority_seeds, &program_key);

        let instructions = [create_auction_instruction(
            *auction_program_key,
            payer.pubkey(),
            CreateAuctionArgs {
                resource: *vault_key,
                authority: auction_manager_key,
                end_auction_at: Some(end_time),
                end_auction_gap: Some(gap_time),
                winners: match winner_limit {
                    0 => WinnerLimit::Unlimited,
                    val => WinnerLimit::Capped(val.try_into().unwrap()),
                },
                token_mint: *payer_mint_key,
            },
        )];

        let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
        let recent_blockhash = client.get_recent_blockhash().unwrap().0;

        transaction.sign(&signers, recent_blockhash);
        client.send_and_confirm_transaction(&transaction).unwrap();
        auction_key = actual_auction_key;
    } else {
        auction_key = pubkey_of(app_matches, "auction").unwrap();
    }

    auction_key
}

fn add_tokens_to_vault_activate_and_return_open_edition_if_existing(
    app_matches: &ArgMatches,
    json_settings: &JSONAuctionManagerSettings,
    vault_key: &Pubkey,
    payer: &Keypair,
    auction_manager_key: &Pubkey,
    client: &RpcClient,
) -> Option<Pubkey> {
    let open_edition_mint_key: Option<Pubkey>;
    if !app_matches.is_present("vault") {
        for n in 0..json_settings.winning_configs.len() {
            let config = json_settings.winning_configs[n].clone();
            add_token_to_vault(
                &payer,
                vault_key,
                &payer,
                client,
                config.amount.into(),
                match config.mint {
                    Some(val) => Some(Pubkey::from_str(&val).unwrap()),
                    None => None,
                },
                match config.account {
                    Some(val) => Some(Pubkey::from_str(&val).unwrap()),
                    None => None,
                },
                match config.edition_type {
                    0 => false,
                    _ => true,
                },
                config.desired_supply,
            );
        }

        if let Some(config) = &json_settings.open_edition_config {
            let (_, actual_open_edition_mint) = add_token_to_vault(
                &payer,
                vault_key,
                &payer,
                client,
                1,
                match &config.mint {
                    Some(val) => Some(Pubkey::from_str(&val).unwrap()),
                    None => None,
                },
                match &config.account {
                    Some(val) => Some(Pubkey::from_str(&val).unwrap()),
                    None => None,
                },
                true,
                None,
            );
            open_edition_mint_key = Some(actual_open_edition_mint);
        } else {
            open_edition_mint_key = None; // Return nothing, it wont be used
        }

        activate_vault(&payer, vault_key, &payer, client);

        combine_vault(&payer, auction_manager_key, vault_key, &payer, client);
    } else {
        open_edition_mint_key = match &json_settings.open_edition_config {
            Some(val) => match &val.mint {
                Some(mint) => Some(Pubkey::from_str(&mint).unwrap()),
                None => None, // If a config was provided for existing vault but no mint, cant do anything here.
            },
            None => None, // Return nothing, it wont be used
        }
    }

    open_edition_mint_key
}

pub fn initialize_auction_manager(
    app_matches: &ArgMatches,
    payer: Keypair,
    client: RpcClient,
) -> (Pubkey, AuctionManager) {
    let program_key = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();
    let vault_program_key = Pubkey::from_str(VAULT_PROGRAM_PUBKEY).unwrap();
    let auction_program_key = Pubkey::from_str(AUCTION_PROGRAM_PUBKEY).unwrap();

    let token_key = Pubkey::from_str(TOKEN_PROGRAM_PUBKEY).unwrap();
    let authority = pubkey_of(app_matches, "authority").unwrap_or_else(|| payer.pubkey());

    let (settings, json_settings) = parse_settings(app_matches.value_of("settings_file").unwrap());

    let vault_key: Pubkey;
    let mut instructions: Vec<Instruction> = vec![];
    let signers: Vec<&Keypair> = vec![&payer];

    let payer_mint_key = Keypair::new();
    let external_keypair = Keypair::new();
    let external_key = find_or_initialize_external_account(
        app_matches,
        &payer,
        &vault_program_key,
        &token_key,
        &client,
        &payer_mint_key,
        &external_keypair,
    );

    // Create vault first, so we can use it to make auction, then add stuff to vault.
    if !app_matches.is_present("vault") {
        vault_key = initialize_vault(&payer, &external_key, &payer, &client);
    } else {
        vault_key = pubkey_of(app_matches, "vault").unwrap();
    }

    let auction_key = find_or_initialize_auction(
        app_matches,
        &vault_key,
        &program_key,
        &auction_program_key,
        &payer_mint_key.pubkey(),
        &payer,
        &client,
    );
    let seeds = &[
        spl_metaplex::state::PREFIX.as_bytes(),
        &auction_key.as_ref(),
    ];
    let (auction_manager_key, _) = Pubkey::find_program_address(seeds, &program_key);

    let open_edition_mint_key = add_tokens_to_vault_activate_and_return_open_edition_if_existing(
        app_matches,
        &json_settings,
        &vault_key,
        &payer,
        &auction_manager_key,
        &client,
    );

    let token_metadata = spl_token_metadata::id();
    let metadata_key: Option<Pubkey>;
    let metadata_authority: Option<Pubkey>;
    let name_symbol_key: Option<Pubkey>;
    let edition_key: Option<Pubkey>;
    let open_edition_master_mint: Option<Pubkey>;
    let open_edition_master_mint_authority: Option<Pubkey>;

    match open_edition_mint_key {
        Some(val) => {
            let metadata_seeds = &[
                spl_token_metadata::state::PREFIX.as_bytes(),
                &token_metadata.as_ref(),
                &val.as_ref(),
            ];
            let (mkey, _) = Pubkey::find_program_address(metadata_seeds, &spl_token_metadata::id());
            let metadata_account = client.get_account(&mkey).unwrap();
            metadata_key = Some(mkey);
            let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

            let name_symbol_seeds = &[
                spl_token_metadata::state::PREFIX.as_bytes(),
                &token_metadata.as_ref(),
                metadata.data.name.as_bytes(),
                metadata.data.symbol.as_bytes(),
            ];
            let (ns_key, _) = Pubkey::find_program_address(name_symbol_seeds, &token_metadata);
            name_symbol_key = Some(ns_key);
            let ns_account = client.get_account(&ns_key);

            match ns_account {
                Ok(acct) => {
                    let ns: NameSymbolTuple = try_from_slice_unchecked(&acct.data).unwrap();
                    metadata_authority = Some(ns.update_authority);
                }
                Err(_) => {
                    metadata_authority = metadata.non_unique_specific_update_authority;
                }
            }

            let edition_seeds = &[
                spl_token_metadata::state::PREFIX.as_bytes(),
                token_metadata.as_ref(),
                val.as_ref(),
                EDITION.as_bytes(),
            ];
            let (ek, _) = Pubkey::find_program_address(edition_seeds, &token_metadata);
            edition_key = Some(ek);
            let master_edition_account = client.get_account(&ek);

            open_edition_master_mint_authority = Some(payer.pubkey());
            match master_edition_account {
                Ok(acct) => {
                    if acct.data[0] == Key::MasterEditionV1 as u8 {
                        let master_edition: MasterEdition =
                            try_from_slice_unchecked(&acct.data).unwrap();
                        open_edition_master_mint = Some(master_edition.master_mint);
                    } else {
                        open_edition_master_mint = None
                    }
                }
                Err(_) => open_edition_master_mint = None,
            }
        }
        None => {
            metadata_key = None;
            metadata_authority = None;
            name_symbol_key = None;
            edition_key = None;
            open_edition_master_mint = None;
            open_edition_master_mint_authority = None;
        }
    }

    instructions.push(create_init_auction_manager_instruction(
        program_key,
        auction_manager_key,
        vault_key,
        auction_key,
        metadata_key,
        name_symbol_key,
        metadata_authority,
        edition_key,
        open_edition_mint_key,
        open_edition_master_mint,
        open_edition_master_mint_authority,
        authority,
        payer.pubkey(),
        vault_program_key,
        auction_program_key,
        settings,
    ));

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    let recent_blockhash = client.get_recent_blockhash().unwrap().0;

    transaction.sign(&signers, recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();
    let account = client.get_account(&auction_manager_key).unwrap();
    let manager: AuctionManager = try_from_slice_unchecked(&account.data).unwrap();

    (auction_manager_key, manager)
}
