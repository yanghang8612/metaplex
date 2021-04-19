use {
    crate::{AUCTION_PROGRAM_PUBKEY, TOKEN_PROGRAM_PUBKEY},
    clap::ArgMatches,
    solana_clap_utils::input_parsers::pubkey_of,
    solana_client::rpc_client::RpcClient,
    solana_program::{account_info::AccountInfo, borsh::try_from_slice_unchecked},
    solana_sdk::{
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
    },
    spl_auction::{
        instruction::place_bid_instruction,
        processor::{place_bid::PlaceBidArgs, AuctionData, BidderMetadata},
    },
    spl_metaplex::{state::AuctionManager, utils::assert_initialized},
    spl_token::{instruction::mint_to, state::Mint},
    std::str::FromStr,
};

pub fn make_bid(app_matches: &ArgMatches, payer: Keypair, client: RpcClient) {
    let auction_program_key = Pubkey::from_str(AUCTION_PROGRAM_PUBKEY).unwrap();
    let token_key = Pubkey::from_str(TOKEN_PROGRAM_PUBKEY).unwrap();

    let wallet = read_keypair_file(
        app_matches
            .value_of("wallet")
            .unwrap_or_else(|| app_matches.value_of("keypair").unwrap()),
    )
    .unwrap();

    let amount = app_matches
        .value_of("price")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let auction_manager_key = pubkey_of(app_matches, "auction_manager").unwrap();

    let account = client.get_account(&auction_manager_key).unwrap();
    let manager: AuctionManager = try_from_slice_unchecked(&account.data).unwrap();

    let auction_account = client.get_account(&manager.auction).unwrap();
    let auction: AuctionData = try_from_slice_unchecked(&auction_account.data).unwrap();
    let mut mint_data = client.get_account(&auction.token_mint).unwrap();
    let data = mint_data.data.as_mut_slice();
    let lamports = &mut 0;
    let mint_info = AccountInfo::new(
        &auction.token_mint,
        false,
        false,
        lamports,
        data,
        &token_key,
        false,
        0,
    );
    let mint: Mint = assert_initialized(&mint_info).unwrap();
    let mut instructions = vec![];

    // Make sure you can afford the bid.

    if app_matches.is_present("mint_it") {
        instructions.push(
            mint_to(
                &token_key,
                &auction.token_mint,
                &wallet.pubkey(),
                &payer.pubkey(),
                &[&payer.pubkey()],
                amount,
            )
            .unwrap(),
        )
    }

    instructions.push(place_bid_instruction(
        auction_program_key,
        wallet.pubkey(),
        wallet.pubkey(),
        auction.token_mint,
        mint.mint_authority.unwrap(),
        PlaceBidArgs {
            amount,
            resource: manager.vault,
        },
    ));

    let signers = [&wallet];
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    let recent_blockhash = client.get_recent_blockhash().unwrap().0;

    transaction.sign(&signers, recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();

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
    let _bid: BidderMetadata = try_from_slice_unchecked(&bidding_metadata.data).unwrap();

    println!("Created bid {:?}", meta_key);
}
