use {
    clap::{crate_description, crate_name, crate_version, App, Arg, ArgMatches, SubCommand},
    initialize_auction_manager::initialize_auction_manager,
    serde::{Deserialize, Serialize},
    serde_json::Result,
    settings_utils::parse_settings,
    solana_clap_utils::{
        input_parsers::pubkey_of,
        input_validators::{is_url, is_valid_pubkey, is_valid_signer},
    },
    solana_client::rpc_client::RpcClient,
    solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack},
    solana_sdk::{
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        system_instruction::create_account,
        transaction::Transaction,
    },
    spl_auction::{
        instruction::create_auction,
        processor::{create_auction::CreateAuctionArgs, WinnerLimit},
    },
    spl_metaplex::{
        instruction::create_init_auction_manager_instruction,
        state::{AuctionManager, AuctionManagerSettings, EditionType, WinningConfig},
    },
    spl_token::{
        instruction::{approve, initialize_account, initialize_mint, mint_to},
        state::{Account, Mint},
    },
    spl_token_metadata::state::EDITION,
    spl_token_vault::{
        instruction::{
            create_activate_vault_instruction, create_add_shares_instruction,
            create_add_token_to_inactive_vault_instruction, create_combine_vault_instruction,
            create_init_vault_instruction, create_mint_shares_instruction,
            create_redeem_shares_instruction, create_update_external_price_account_instruction,
            create_withdraw_shares_instruction, create_withdraw_tokens_instruction,
        },
        state::{
            ExternalPriceAccount, SafetyDepositBox, MAX_EXTERNAL_ACCOUNT_SIZE, MAX_VAULT_SIZE,
        },
    },
    std::{convert::TryInto, str::FromStr},
    vault_utils::{activate_vault, add_token_to_vault, combine_vault, initialize_vault},
};
mod initialize_auction_manager;
mod settings_utils;
mod vault_utils;

pub const VAULT_PROGRAM_PUBKEY: &str = "94wRaYAQdC2gYF76AUTYSugNJ3rAC4EimjAMPwM7uYry";
pub const AUCTION_PROGRAM_PUBKEY: &str = "94wRaYAQdC2gYF76AUTYSugNJ3rAC4EimjAMPwM7uYry";

pub const PROGRAM_PUBKEY: &str = "94wRaYAQdC2gYF76AUTYSugNJ3rAC4EimjAMPwM7uYry";

pub const TOKEN_PROGRAM_PUBKEY: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

fn main() {
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("keypair")
                .long("keypair")
                .value_name("KEYPAIR")
                .validator(is_valid_signer)
                .takes_value(true)
                .global(true)
                .help("Filepath or URL to a keypair"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(is_url)
                .help("JSON RPC URL for the cluster [default: devnet]"),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize an Auction Manager")
                .arg(
                    Arg::with_name("authority")
                        .long("authority")
                        .value_name("AUTHORITY")
                        .required(false)
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .help("Pubkey of authority, defaults to you otherwise"),
                )
                .arg(
                    Arg::with_name("external_price_account")
                        .long("external_price_account")
                        .value_name("EXTERNAL_PRICE_ACCOUNT")
                        .required(true)
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .help("Pubkey of external price account, if one not provided, one will be made. Needs to be same as the one on the Vault."),
                )
                .arg(
                    Arg::with_name("vault")
                        .long("vault")
                        .value_name("VAULT")
                        .required(true)
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .help("Pubkey of vault. If one not provided, one will be made."),
                )
                .arg(
                    Arg::with_name("auction")
                        .long("auction")
                        .value_name("AUCTION")
                        .required(true)
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .help("Pubkey of auction. If one not provided, one will be made."),
                )
                .arg(
                    Arg::with_name("winner_limit")
                        .long("winner_limit")
                        .value_name("WINNER_LIMIT")
                        .required(false)
                        .takes_value(true)
                        .help("Defaults to unlimited (0), ignored if existing auction provided."),
                ).arg(
                    Arg::with_name("gap_time")
                        .long("gap_time")
                        .value_name("GAP_TIME")
                        .required(false)
                        .takes_value(true)
                        .help("Defaults to 1200 slots, ignored if existing auction provided."),
                )
                .arg(
                    Arg::with_name("settings_file")
                        .long("settings_file")
                        .value_name("SETTINGS_FILE")
                        .takes_value(false)
                        .required(true)
                        .help("File path or uri to settings file (json) for setting up Auction Managers. See settings_sample.json, and you can follow the JSON structs in settings_utils.rs to customize the AuctionManagerSetting struct that gets created for shipping."),
                ),
        )
        .get_matches();

    let client = RpcClient::new(
        app_matches
            .value_of("json_rpc_url")
            .unwrap_or(&"https://devnet.solana.com".to_owned())
            .to_owned(),
    );

    let (sub_command, sub_matches) = app_matches.subcommand();

    let payer = read_keypair_file(app_matches.value_of("keypair").unwrap()).unwrap();

    match (sub_command, sub_matches) {
        ("init", Some(arg_matches)) => {
            let (key, manager) = initialize_auction_manager(arg_matches, payer, client);
            println!(
                "Created auction manager with address {:?} and output {:?}",
                key, manager
            );
        }

        _ => unreachable!(),
    }
}
