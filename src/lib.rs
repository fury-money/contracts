// lib.rs

pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

// Re-export necessary dependencies from cosmwasm_std
pub use cosmwasm_std::{
    Addr, Api, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, HandleResponse, HumanAddr, InitResponse,
    MessageInfo, QueryRequest, QueryResponse, ReadonlyStorage, StdError, StdResult, Storage, Uint128, WasmMsg,
};

// Re-export types and functions from cw721 and cw721_base
pub use cw721::{ContractInfoResponse, MinterResponse, OwnerOfResponse, TokenInfoResponse};
pub use cw721_base::msg::{ExecuteMsg as NftExecuteMsg, InstantiateMsg as NftInstantiateMsg, QueryMsg as NftQueryMsg};
