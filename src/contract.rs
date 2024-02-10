use cosmwasm_std::{
    Addr, Api, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, InitResponse, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use std::cmp;
use cw721::{MinterResponse, TokenInfoResponse, ContractInfoResponse, OwnerOfResponse};
use cw721_base::msg::InstantiateMsg as NftInstantiateMsg;
use cw721_base::msg::ExecuteMsg as NftExecuteMsg;
use cw721_base::msg::QueryMsg as NftQueryMsg;

// Constants
const CONFIG_KEY: &[u8] = b"config";
const BREED_COUNT_KEY: &[u8] = b"breed_count";
const BREEDS_KEY: &[u8] = b"breeds";

// Initialization function
pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<InitResponse> {
    let config_state = ConfigState {
        config: Some(config.clone()),
        owner: deps.api.canonical_address(&info.sender)?,
    };
    save_config(deps.storage, &config_state)?;

    let breed_count = BreedCount {
        count: 0,
        latest_id: 0,
    };
    save_breed_count(deps.storage, &breed_count)?;

    Ok(InitResponse::default())
}

// Handle messages function
pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig { breed_count_limit, breed_duration, breed_price_amount, breed_price_denom, owner } => update_config(deps, env, info, breed_count_limit, breed_duration, breed_price_amount, breed_price_denom, owner),
        HandleMsg::StartBreed {} => start_breed(deps, env, info),
        HandleMsg::Breed { nft_token_id1, nft_token_id2 } => breed(deps, env, info, nft_token_id1, nft_token_id2),
        HandleMsg::Mint { extension, token_id, token_uri } => mint(deps, env, info, extension, token_id, token_uri),
        HandleMsg::Withdraw { breed_id } => withdraw(deps, env, info, breed_id),
        HandleMsg::WithdrawFund {} => withdraw_fund(deps, env, info),
    }
}

// Query function
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::BreedInfo { breed_id } => to_binary(&query_breed_info(deps, breed_id)?),
        QueryMsg::BreededCount { parent_nft_token_id } => to_binary(&query_breeded_count(deps, parent_nft_token_id)?),
        QueryMsg::BreedRequestsCount {} => to_binary(&query_breed_requests_count(deps)?),
        QueryMsg::BreedFinishedCount {} => to_binary(&query_breed_finished_count(deps)?),
        QueryMsg::QueryBreedings { count, from, sort } => to_binary(&query_breedings(deps, count, from, sort)?),
        QueryMsg::QueryBreedingsLength {} => to_binary(&query_breedings_length(deps)?),
        QueryMsg::QueryUserBreedings { count, from, sort, user } => to_binary(&query_user_breedings(deps, count, from, sort, user)?),
        QueryMsg::QueryUserBreedingsLength { user } => to_binary(&query_user_breedings_length(deps, user)?),
    }
}

// Function to save config state
fn save_config(storage: &mut dyn Storage, config: &ConfigState) -> StdResult<()> {
    singleton(storage, CONFIG_KEY).save(config)
}

// Function to read config state
fn read_config(storage: &dyn Storage) -> StdResult<ConfigState> {
    singleton_read(storage, CONFIG_KEY).load()
}

// Function to save breed count
fn save_breed_count(storage: &mut dyn Storage, breed_count: &BreedCount) -> StdResult<()> {
    singleton(storage, BREED_COUNT_KEY).save(breed_count)
}

// Function to read breed count
fn read_breed_count(storage: &dyn Storage) -> StdResult<BreedCount> {
    singleton_read(storage, BREED_COUNT_KEY).load()
}

// Function to save breed
fn save_breed(storage: &mut dyn Storage, id: u64, breed: &Breed) -> StdResult<()> {
    let mut key = id.to_be_bytes().to_vec();
    key.insert(0, BREEDS_KEY);
    storage.set(&key, &to_vec(breed)?);
    Ok(())
}

// Function to read breed
fn read_breed(storage: &dyn Storage, id: u64) -> StdResult<Option<Breed>> {
    let mut key = id.to_be_bytes().to_vec();
    key.insert(0, BREEDS_KEY);
    match storage.get(&key) {
        Some(data) => Ok(Some(from_slice(&data)?)),
        None => Ok(None),
    }
}

// Function to remove breed
fn remove_breed(storage: &mut dyn Storage, id: u64) -> StdResult<()> {
    let mut key = id.to_be_bytes().to_vec();
    key.insert(0, BREEDS_KEY);
    storage.remove(&key);
    Ok(())
}

// Function to update config
fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    breed_count_limit: Option<u32>,
    breed_duration: Option<u64>,
    breed_price_amount: Option<Uint128>,
    breed_price_denom: Option<String>,
    owner: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let mut config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    if let Some(limit) = breed_count_limit {
        config.config.breed_count_limit = limit;
    }
    if let Some(duration) = breed_duration {
        config.config.breed_duration = duration;
    }
    if let Some(amount) = breed_price_amount {
        config.config.breed_price_amount = amount;
    }
    if let Some(denom) = breed_price_denom {
        config.config.breed_price_denom = denom;
    }
    if let Some(new_owner) = owner {
        config.owner = deps.api.canonical_address(&new_owner)?;
    }
    save_config(deps.storage, &config)?;
    Ok(HandleResponse::default())
}

// Other functions such as start_breed, breed, mint, withdraw, withdraw_fund, query_config, query_breed_info, query_breeded_count, query_breed_requests_count, query_breed_finished_count, query_breedings, query_breedings_length, query_user_breedings, and sort_breedings go here...


fn start_breed(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<HandleResponse> {
    let config = read_config(deps.storage)?;
    let breed_count = read_breed_count(deps.storage)?;

    if breed_count.count >= config.config.breed_count_limit as u64 {
        return Err(StdError::generic_err("Maximum breed count reached"));
    }

    let breed_id = breed_count.latest_id + 1;
    let new_breed = Breed {
        id: breed_id,
        start_time: env.block.time,
        nft_owner: deps.api.canonical_address(&info.sender)?,
        nft_token_id1: String::default(), // Provide mechanism to acquire parent NFTs
        nft_token_id2: String::default(), // Provide mechanism to acquire parent NFTs
        end_time: env.block.time + config.config.breed_duration,
        withdrawn: false,
    };

    save_breed(deps.storage, breed_id, &new_breed)?;
    let updated_breed_count = BreedCount {
        count: breed_count.count + 1,
        latest_id: breed_id,
    };
    save_breed_count(deps.storage, &updated_breed_count)?;

    Ok(HandleResponse::default())
}

fn breed(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft_token_id1: String,
    nft_token_id2: String,
) -> StdResult<HandleResponse> {
    let config = read_config(deps.storage)?;
    let breed_count = read_breed_count(deps.storage)?;

    let breed_id = breed_count.latest_id + 1;
    let new_breed = Breed {
        id: breed_id,
        start_time: env.block.time,
        nft_owner: deps.api.canonical_address(&info.sender)?,
        nft_token_id1,
        nft_token_id2,
        end_time: env.block.time + config.config.breed_duration,
        withdrawn: false,
    };

    save_breed(deps.storage, breed_id, &new_breed)?;
    let updated_breed_count = BreedCount {
        count: breed_count.count + 1,
        latest_id: breed_id,
    };
    save_breed_count(deps.storage, &updated_breed_count)?;

    Ok(HandleResponse::default())
}

fn mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    extension: Option<Metadata>,
    token_id: String,
    token_uri: Option<String>,
) -> StdResult<HandleResponse> {
    // Get the contract address of the NFT contract
    let nft_contract_address = deps.api.addr_humanize(&config.child_contract_addr)?;

    // Generate the NFT instantiate message
    let nft_instantiate_msg = NftInstantiateMsg {
        name: "My NFT".to_string(),
        symbol: "NFT".to_string(),
        minter: env.contract.address.clone(),
    };

    // Execute the NFT instantiate message
    let execute_msg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: 123, // Replace with the actual code ID of the NFT contract
        send: vec![],
        label: "nft".to_string(),
        msg: to_binary(&nft_instantiate_msg)?,
        funds: vec![],
    });

    // Dispatch the execute message
    let res = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: nft_contract_address.clone(),
        msg: to_binary(&NftQueryMsg::ContractInfo {})?,
    }))?;

    // Check if the NFT contract is already instantiated
    let contract_info: ContractInfoResponse = from_binary(&res)?;

    // If NFT contract is not already instantiated, instantiate it
    if !contract_info.is_initialized() {
        execute(deps.as_mut(), env.clone(), info.clone(), execute_msg)?;
    }

    // Generate the mint NFT execute message
    let nft_mint_msg = NftExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: env.message.sender.clone(),
        uri: token_uri.clone(),
        name: None,
        description: None,
        image: None,
    };

    // Execute the mint NFT execute message
    let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nft_contract_address,
        msg: to_binary(&nft_mint_msg)?,
        send: vec![],
    });

    // Dispatch the execute message
    execute(deps.as_mut(), env, info, execute_msg)
}

fn withdraw(deps: DepsMut, env: Env, info: MessageInfo, breed_id: u64) -> StdResult<HandleResponse> {
    let config = read_config(deps.storage)?;
    let mut breed = read_breed(deps.storage, breed_id)?;
    let sender_address = deps.api.canonical_address(&info.sender)?;

    if breed.is_none() {
        return Err(StdError::NotFound { kind: "Breed".to_string() });
    }

    let breed = breed.unwrap();

    if sender_address != breed.nft_owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if env.block.time < breed.end_time {
        return Err(StdError::generic_err("Breed process has not yet finished"));
    }

    if breed.withdrawn {
        return Err(StdError::generic_err("Breed process has already been withdrawn"));
    }

    // Logic for withdrawing NFTs and handling funds

    breed.withdrawn = true;
    save_breed(deps.storage, breed_id, &breed)?;

    Ok(HandleResponse::default())
}

fn withdraw_fund(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<HandleResponse> {
    // Ensure only the contract owner can withdraw funds
    if info.sender != config.owner {
        return Err(StdError::Unauthorized {});
    }

    // Get the balance of the contract
    let contract_balance = deps.querier.query_balance(&env.contract.address)?;

    // Ensure the contract has sufficient funds to withdraw
    if contract_balance.amount.is_zero() {
        return Err(StdError::generic_err("Contract balance is zero"));
    }

    // Specify the recipient to transfer the funds to
    let recipient = info.sender.clone(); // Replace with the actual recipient address

    // Generate the execute message to transfer funds
    let execute_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![coin(contract_balance.amount, contract_balance.denom.clone())],
    });

    // Dispatch the execute message
    execute(deps, env, info, execute_msg)
}

fn query_config(deps: Deps) -> ConfigResponse {
    let config = read_config(deps.storage).unwrap();
    ConfigResponse {
        breed_count_limit: config.config.breed_count_limit,
        breed_duration: config.config.breed_duration,
        breed_price_amount: config.config.breed_price_amount,
        breed_price_denom: config.config.breed_price_denom,
        breed_start_time: config.config.breed_start_time,
        child_base_uri: config.config.child_base_uri,
        child_contract_addr: deps.api.human_address(&config.config.child_contract_addr).unwrap(),
        child_nft_max_supply: config.config.child_nft_max_supply,
        owner: deps.api.human_address(&config.owner).unwrap(),
        parent_contract_addr: deps.api.human_address(&config.config.parent_contract_addr).unwrap(),
    }
}

fn query_breed_info(deps: Deps, breed_id: u64) -> BreedInfoResponse {
    let breed = read_breed(deps.storage, breed_id).unwrap().unwrap();
    BreedInfoResponse {
        child_token_id: None, // Provide mechanism to query child NFT details
        end_time: breed.end_time,
        nft_owner: deps.api.human_address(&breed.nft_owner).unwrap(),
        nft_token_id1: breed.nft_token_id1,
        nft_token_id2: breed.nft_token_id2,
        start_time: breed.start_time,
        withdrawn: breed.withdrawn,
    }
}

fn query_breed_requests_count(deps: Deps) -> BreedRequestsCountResponse {
    // Load the contract state
    let state = config_read(deps.storage).load().unwrap_or_default();

    // Access the breed requests count from the contract state
    state.breed_requests_count
}

fn query_breed_finished_count(deps: Deps) -> BreedFinishedCountResponse {
    // Load the contract state
    let state = config_read(deps.storage).load().unwrap_or_default();

    // Access the breed finished count from the contract state
    state.breed_finished_count
}

fn query_breeded_count(deps: Deps, parent_nft_token_id: String) -> BreededCountResponse {
    // Load the contract state
    let state = config(deps.storage).load().unwrap_or_default();

    // Access the breeded count for the specified parent NFT token ID from the contract state
    // Assuming breeded_counts is a map in your contract state storing the breeded count for each parent NFT token ID
    let breeded_counts = state.breeded_counts;

    // Get the breeded count for the specified parent NFT token ID
    let breeded_count = breeded_counts.get(&parent_nft_token_id).cloned().unwrap_or_default();

    breeded_count
}

fn query_breedings(
    deps: Deps,
    count: u32,
    from: u32,
    sort: String,
) -> QueryBreedingsResponse {
    // Load the contract state or query from external storage if needed
    let state = config_read(deps.storage).load().unwrap_or_default();

    // Retrieve all breedings from the contract state or external storage
    let all_breedings: Vec<BreedInfo> = unimplemented!(); // Replace with your logic to retrieve all breedings

    // Sort the breedings based on the provided sort criteria
    let sorted_breedings = sort_breedings(all_breedings, &sort);

    // Slice the breedings based on the provided count and from parameters
    let start_index = from as usize;
    let end_index = (from + count) as usize;
    let sliced_breedings = sorted_breedings[start_index..end_index].to_vec();

    // Return the sliced breedings as the response
    sliced_breedings
}

fn sort_breedings(breedings: Vec<BreedInfo>, sort: &str) -> Vec<BreedInfo> {
    let mut sorted_breedings = breedings;

    // Sort the breedings based on the provided criteria
    match sort.as_str() {
        "ascending" => {
            sorted_breedings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        }
        "descending" => {
            sorted_breedings.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        }
        _ => {
            // Default sorting logic if criteria is not recognized
            // For example, you might sort by default based on the start_time in ascending order
            sorted_breedings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        }
    }

    sorted_breedings
}

fn query_breedings_length(deps: Deps) -> QueryBreedingsLengthResponse {
    // Load the contract state or query from external storage if needed
    let state = config_read(deps.storage).load().unwrap_or_default();

    // Access the length of breed processes from the contract state
    let breedings_length = state.breedings.len() as u32;

    breedings_length
}

fn query_user_breedings(
    deps: Deps,
    count: u32,
    from: u32,
    sort: String,
    user: HumanAddr,
) -> QueryUserBreedingsResponse {
    // Load the contract state or query from external storage if needed
    let state = config_read(deps.storage).load().unwrap_or_default();

    // Retrieve all breed processes associated with the specified user
    let user_breedings: Vec<BreedInfo> = state
        .breedings
        .iter()
        .filter(|breeding| breeding.user == user)
        .cloned()
        .collect();

    // Sort the breedings based on the provided sort criteria
    let sorted_breedings = sort_breedings(user_breedings, &sort);

    // Slice the breedings based on the provided count and from parameters
    let start_index = from as usize;
    let end_index = (from + count) as usize;
    let sliced_breedings = sorted_breedings[start_index..end_index].to_vec();

    // Return the sliced breedings as the response
    sliced_breedings
}

fn sort_breedings(breedings: Vec<BreedInfo>, sort: &str) -> Vec<BreedInfo> {
    let mut sorted_breedings = breedings;

    // Sort the breedings based on the provided criteria
    match sort.as_str() {
        "ascending" => {
            sorted_breedings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        }
        "descending" => {
            sorted_breedings.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        }
        _ => {
            // Default sorting logic if criteria is not recognized
            // For example, you might sort by default based on the start_time in ascending order
            sorted_breedings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        }
    }

    sorted_breedings
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    #[prost(uint32, tag = "1")]
    pub breed_count_limit: u32,
    #[prost(uint64, tag = "2")]
    pub breed_duration: u64,
    #[prost(string, tag = "3")]
    pub breed_price_amount: String,
    #[prost(string, tag = "4")]
    pub breed_price_denom: String,
    #[prost(uint64, tag = "5")]
    pub breed_start_time: u64,
    #[prost(string, tag = "6")]
    pub child_base_uri: String,
    #[prost(string, tag = "7")]
    pub child_contract_addr: String,
    #[prost(uint32, tag = "8")]
    pub child_nft_max_supply: u32,
    #[prost(string, tag = "9")]
    pub owner: String,
    #[prost(string, tag = "10")]
    pub parent_contract_addr: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Breed {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(uint64, tag = "2")]
    pub start_time: u64,
    #[prost(string, tag = "3")]
    pub nft_owner: String,
    #[prost(string, tag = "4")]
    pub nft_token_id1: String,
    #[prost(string, tag = "5")]
    pub nft_token_id2: String,
    #[prost(uint64, tag = "6")]
    pub end_time: u64,
    #[prost(bool, tag = "7")]
    pub withdrawn: bool,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BreedCount {
    #[prost(uint64, tag = "1")]
    pub count: u64,
    #[prost(uint64, tag = "2")]
    pub latest_id: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigState {
    #[prost(message, optional, tag = "1")]
    pub config: Option<Config>,
    #[prost(string, tag = "2")]
    pub owner: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BreedInfoResponse {
    #[prost(string, tag = "1")]
    pub child_token_id: String,
    #[prost(uint64, tag = "2")]
    pub end_time: u64,
    #[prost(string, tag = "3")]
    pub nft_owner: String,
    #[prost(string, tag = "4")]
    pub nft_token_id1: String,
    #[prost(string, tag = "5")]
    pub nft_token_id2: String,
    #[prost(uint64, tag = "6")]
    pub start_time: u64,
    #[prost(bool, tag = "7")]
    pub withdrawn: bool,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBreedingsResponse {
    #[prost(message, repeated, tag = "1")]
    pub breed_info: Vec<BreedInfoResponse>,
}

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

pub type BreedFinishedCountResponse = u32;
pub type Addr = String;
pub type BreedRequestsCountResponse = u32;
pub type BreededCountResponse = u32;
pub type Uint128 = String;
pub type QueryBreedingsLengthResponse = u32;
pub type QueryUserBreedingsLengthResponse = u32;
pub type QueryUserBreedingsResponse = Vec<BreedInfoResponse>;
pub type Attribute = Vec<u8>;
pub type Metadata = Vec<u8>;
