use {
    serde::{Deserialize, Serialize},
    spl_metaplex::state::{
        AuctionManagerSettings, EditionType, NonWinningConstraint, WinningConfig, WinningConstraint,
    },
    std::fs::File,
};
#[derive(Serialize, Deserialize, Clone)]
pub struct JSONWinningConfig {
    pub safety_deposit_box_index: u8,
    pub amount: u8,
    pub edition_type: u8,
    pub desired_supply: Option<u64>,
    pub mint: Option<String>,
    pub account: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct JSONOpenEditionConfig {
    pub safety_deposit_box_index: u8,
    pub mint: Option<String>,
    pub account: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct JSONAuctionManagerSettings {
    pub open_edition_winner_constraint: u8,

    pub open_edition_non_winning_constraint: u8,
    pub winning_configs: Vec<JSONWinningConfig>,

    pub open_edition_config: Option<JSONOpenEditionConfig>,

    pub open_edition_fixed_price: Option<u64>,
}

pub fn parse_settings(settings_file: &str) -> (AuctionManagerSettings, JSONAuctionManagerSettings) {
    let file = File::open(settings_file).unwrap();
    let json_settings: JSONAuctionManagerSettings = serde_json::from_reader(file).unwrap();
    let mut parsed_winning_configs: Vec<WinningConfig> = vec![];

    for n in 0..json_settings.winning_configs.len() {
        let json_box = json_settings.winning_configs[n].clone();
        parsed_winning_configs.push(WinningConfig {
            safety_deposit_box_index: json_box.safety_deposit_box_index,
            amount: json_box.amount,
            edition_type: match json_box.edition_type {
                0 => EditionType::NA,
                1 => EditionType::MasterEdition,
                2 => EditionType::LimitedEdition,
                _ => EditionType::NA,
            },
        })
    }

    let settings = AuctionManagerSettings {
        winning_configs: parsed_winning_configs,
        open_edition_winner_constraint: match json_settings.open_edition_winner_constraint {
            0 => WinningConstraint::NoOpenEdition,
            1 => WinningConstraint::OpenEditionGiven,
            _ => WinningConstraint::NoOpenEdition,
        },
        open_edition_non_winning_constraint: match json_settings.open_edition_non_winning_constraint
        {
            0 => NonWinningConstraint::NoOpenEdition,
            1 => NonWinningConstraint::GivenForFixedPrice,
            2 => NonWinningConstraint::GivenForBidPrice,
            _ => NonWinningConstraint::NoOpenEdition,
        },
        open_edition_config: match &json_settings.open_edition_config {
            Some(val) => Some(val.safety_deposit_box_index),
            None => None,
        },
        open_edition_fixed_price: json_settings.open_edition_fixed_price,
    };

    (settings, json_settings)
}
