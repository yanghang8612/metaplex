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
        instruction::create_auction,
        processor::{create_auction::CreateAuctionArgs, WinnerLimit},
    },
    spl_metaplex::{instruction::create_init_auction_manager_instruction, state::AuctionManager},
    spl_token::{instruction::initialize_mint, state::Mint},
    spl_token_metadata::state::EDITION,
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
    instructions: &mut Vec<Instruction>,
    signers: &'a mut Vec<&'a Keypair>,
    client: &RpcClient,
    payer_mint_key: &'a Keypair,
    external_keypair: &'a Keypair,
) -> (Pubkey, Vec<&'a Keypair>) {
    let external_key: Pubkey;
    if !app_matches.is_present("external_price_account") {
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
        external_key = external_keypair.pubkey();
    } else {
        external_key = pubkey_of(app_matches, "external_price_account").unwrap();
    }

    let mut new_signers: Vec<&Keypair> = vec![];
    for n in 0..signers.len() {
        new_signers.push(signers[n]);
    }
    (external_key, new_signers)
}

fn find_or_initialize_auction(
    app_matches: &ArgMatches,
    program_key: &Pubkey,
    vault_key: &Pubkey,
    auction_program_key: &Pubkey,
    instructions: &mut Vec<Instruction>,
) -> Pubkey {
    let auction_key: Pubkey;
    if !app_matches.is_present("auction") {
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
        instructions.push(create_auction(
            *auction_program_key,
            *program_key,
            CreateAuctionArgs {
                resource: *vault_key,
                end_time: Some(end_time.try_into().unwrap()),
                gap_time: Some(gap_time.try_into().unwrap()),
                winners: match winner_limit {
                    0 => WinnerLimit::Unlimited,
                    val => WinnerLimit::Capped(val.try_into().unwrap()),
                },
            },
        ));

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
) -> Pubkey {
    let open_edition_mint_key: Pubkey;
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
            open_edition_mint_key = actual_open_edition_mint;
        } else {
            open_edition_mint_key = solana_program::system_program::id(); // Return nothing, it wont be used
        }

        activate_vault(&payer, vault_key, &payer, client);

        combine_vault(&payer, auction_manager_key, vault_key, &payer, client);
    } else {
        open_edition_mint_key = match &json_settings.open_edition_config {
            Some(val) => match &val.mint {
                Some(mint) => Pubkey::from_str(&mint).unwrap(),
                None => solana_program::system_program::id(), // If a config was provided for existing vault but no mint, cant do anything here.
            },
            None => solana_program::system_program::id(), // Return nothing, it wont be used
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

    let (settings, json_settings) = parse_settings(app_matches.value_of("settings").unwrap());

    let vault_key: Pubkey;
    let mut instructions: Vec<Instruction> = vec![];
    let mut signers: Vec<&Keypair> = vec![&payer];

    let payer_mint_key = Keypair::new();
    let external_keypair = Keypair::new();
    let (external_key, signers) = find_or_initialize_external_account(
        app_matches,
        &payer,
        &vault_program_key,
        &token_key,
        &mut instructions,
        &mut signers,
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
        &program_key,
        &vault_key,
        &auction_program_key,
        &mut instructions,
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
    let edition_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        token_metadata.as_ref(),
        open_edition_mint_key.as_ref(),
        EDITION.as_bytes(),
    ];
    let (edition_key, _) = Pubkey::find_program_address(edition_seeds, &spl_token_metadata::id());

    instructions.push(create_init_auction_manager_instruction(
        program_key,
        auction_manager_key,
        vault_key,
        auction_key,
        external_key,
        edition_key,
        open_edition_mint_key,
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
