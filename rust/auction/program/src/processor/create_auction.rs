use std::str::FromStr;

use solana_program::clock::UnixTimestamp;

use crate::{
    errors::AuctionError,
    processor::{
        AuctionData, AuctionState, Bid, BidState, BuyNowData, PriceFloor, WinnerLimit,
        BASE_AUCTION_DATA_SIZE, BUY_NOW_DATA_LEN,
    },
    utils::{assert_owned_by, create_or_allocate_account_raw},
    BUY_NOW, PREFIX,
};

use super::EXCLUSIVE_AUCTION_AUTHORITY;

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        clock::Slot,
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    std::mem,
};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CreateAuctionArgs {
    /// How many winners are allowed for this auction. See AuctionData.
    pub winners: WinnerLimit,
    /// End time is the cut-off point that the auction is forced to end by. See AuctionData.
    pub end_auction_at: Option<UnixTimestamp>,
    /// Gap time is how much time after the previous bid where the auction ends. See AuctionData.
    pub end_auction_gap: Option<UnixTimestamp>,
    /// Token mint for the SPL token used for bidding.
    pub token_mint: Pubkey,
    /// The resource being auctioned. See AuctionData.
    pub resource: Pubkey,
    /// Set a price floor.
    pub price_floor: PriceFloor,
    /// Max price of the auction i.e buy now price
    pub max_price: Option<u64>,
}

struct Accounts<'a, 'b: 'a> {
    auction: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
    system: &'a AccountInfo<'b>,
    authority: &'a AccountInfo<'b>,
    buy_now: Option<&'a AccountInfo<'b>>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let account_iter = &mut accounts.iter();
    let accounts = Accounts {
        payer: next_account_info(account_iter)?,
        auction: next_account_info(account_iter)?,
        rent: next_account_info(account_iter)?,
        system: next_account_info(account_iter)?,
        authority: next_account_info(account_iter)?,
        buy_now: next_account_info(account_iter).ok(),
    };
    if !accounts.authority.is_signer {
        msg!("The authority account should be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    let exclusive_auth = Pubkey::from_str(EXCLUSIVE_AUCTION_AUTHORITY).unwrap();
    if &exclusive_auth != accounts.authority.owner {
        msg!("This command can only be called by the Bonfida name auctioning smart contract");
    }
    Ok(accounts)
}

pub fn create_auction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateAuctionArgs,
) -> ProgramResult {
    msg!("+ Processing CreateAuction");
    let accounts = parse_accounts(program_id, accounts)?;

    let auction_path = [
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &args.resource.to_bytes(),
    ];

    // Derive the address we'll store the auction in, and confirm it matches what we expected the
    // user to provide.
    let (auction_key, bump) = Pubkey::find_program_address(&auction_path, program_id);
    if auction_key != *accounts.auction.key {
        return Err(AuctionError::InvalidAuctionAccount.into());
    }
    // The data must be large enough to hold at least the number of winners.
    let auction_size = match args.winners {
        WinnerLimit::Capped(n) => mem::size_of::<Bid>() * n + BASE_AUCTION_DATA_SIZE,
        WinnerLimit::Unlimited(_) => BASE_AUCTION_DATA_SIZE,
    };

    let bid_state = match args.winners {
        WinnerLimit::Capped(n) => BidState::new_english(n),
        WinnerLimit::Unlimited(_) => BidState::new_open_edition(),
    };

    if accounts.auction.data_is_empty() {
        // Create auction account with enough space for a winner tracking.
        create_or_allocate_account_raw(
            *program_id,
            accounts.auction,
            accounts.rent,
            accounts.system,
            accounts.payer,
            auction_size,
            &[
                PREFIX.as_bytes(),
                program_id.as_ref(),
                &args.resource.to_bytes(),
                &[bump],
            ],
        )?;
    } else {
        let parsed: AuctionData = try_from_slice_unchecked(&accounts.auction.data.borrow())?;
        if &parsed.authority != accounts.authority.key {
            msg!("Invalid authority account for already existing auction");
            return Err(ProgramError::InvalidArgument);
        }
    }

    // Allow buy now
    let mut is_buy_now = args.max_price.is_some() && accounts.buy_now.is_some();
    if is_buy_now {
        let max_price = args.max_price.unwrap();
        let buy_now_account = accounts.buy_now.unwrap();

        let buy_now_path = [
            BUY_NOW.as_bytes(),
            program_id.as_ref(),
            &args.resource.to_bytes(),
        ];

        let (buy_now_key, buy_now_bump) = Pubkey::find_program_address(&buy_now_path, program_id);

        if buy_now_account.key != &buy_now_key {
            msg!("Invalid buy now account provided");
            return Err(ProgramError::InvalidArgument);
        }

        if buy_now_account.data_is_empty() {
            create_or_allocate_account_raw(
                *program_id,
                buy_now_account,
                accounts.rent,
                accounts.system,
                accounts.payer,
                BUY_NOW_DATA_LEN,
                &[
                    BUY_NOW.as_bytes(),
                    program_id.as_ref(),
                    &args.resource.to_bytes(),
                    &[buy_now_bump],
                ],
            )?;
        }
        BuyNowData { max_price }.serialize(&mut *buy_now_account.data.borrow_mut())?;
    }

    // Configure Auction.
    AuctionData {
        authority: *accounts.authority.key,
        bid_state: bid_state,
        resource: args.resource,
        end_auction_at: args.end_auction_at,
        end_auction_gap: args.end_auction_gap,
        ended_at: None,
        last_bid: None,
        price_floor: args.price_floor,
        state: AuctionState::create(is_buy_now),
        token_mint: args.token_mint,
    }
    .serialize(&mut *accounts.auction.data.borrow_mut())?;

    Ok(())
}
