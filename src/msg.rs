// msg.rs

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        breed_count_limit: Option<u32>,
        breed_duration: Option<u64>,
        breed_price_amount: Option<Uint128>,
        breed_price_denom: Option<String>,
        owner: Option<HumanAddr>,
    },
    StartBreed {},
    Breed {
        nft_token_id1: String,
        nft_token_id2: String,
    },
    Mint {
        extension: Option<Metadata>,
        token_id: String,
        token_uri: Option<String>,
    },
    Withdraw {
        breed_id: u64,
    },
    WithdrawFund {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BreedInfo {
        breed_id: u64,
    },
    BreededCount {
        parent_nft_token_id: String,
    },
    BreedRequestsCount {},
    BreedFinishedCount {},
    QueryBreedings {
        count: u32,
        from: u32,
        sort: String,
    },
    QueryBreedingsLength {},
    QueryUserBreedings {
        count: u32,
        from: u32,
        sort: String,
        user: HumanAddr,
    },
    QueryUserBreedingsLength {
        user: HumanAddr,
    },
}

// Define Metadata type if not already defined in the contract.rs file
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Metadata(pub Vec<u8>);
