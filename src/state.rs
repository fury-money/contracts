use cosmwasm_std::{CanonicalAddr, Uint128};
use cw721_base::ContractInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub breed_count_limit: u32,
    pub breed_duration: u64,
    pub breed_price_amount: Uint128,
    pub breed_price_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigState {
    pub config: Option<Config>,
    pub owner: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BreedCount {
    pub count: u64,
    pub latest_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Breed {
    pub id: u64,
    pub owner: CanonicalAddr,
    pub nft_token_id1: String,
    pub nft_token_id2: String,
    pub start_time: u64,
    pub end_time: u64,
    pub is_withdrawn: bool,
}

impl Breed {
    pub fn new(id: u64, owner: CanonicalAddr, nft_token_id1: String, nft_token_id2: String, start_time: u64, end_time: u64, is_withdrawn: bool) -> Self {
        Breed {
            id,
            owner,
            nft_token_id1,
            nft_token_id2,
            start_time,
            end_time,
            is_withdrawn,
        }
    }
}
