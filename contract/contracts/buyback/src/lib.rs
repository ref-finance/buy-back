use std::fmt;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey,
    Gas, PanicOnDefault, require, serde_json, PromiseOrValue
};

mod action;
mod errors;
mod owner;
mod token_receiver;
mod view;
mod utils;

pub use action::*;
pub use errors::*;
pub use owner::*;
pub use token_receiver::*;
pub use view::*;
pub use utils::*;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeys {
    TokenWhiteList,
    Guardian
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum RunningState {
    Running, Paused
}

impl fmt::Display for RunningState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunningState::Running => write!(f, "Running"),
            RunningState::Paused => write!(f, "Paused"),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractData {
    pub owner_id: AccountId,
    pub ref_exchange_id: AccountId,
    pub burn_account_id: AccountId, 
    pub company_account_id: AccountId, 
    pub reward_account_id: AccountId,
    pub buyback_token_id: AccountId,
    pub token_white_list: UnorderedSet<AccountId>,
    pub guardians: UnorderedSet<AccountId>,
    pub burn_rate: u32,
    pub company_rate: u32,
    pub reward_rate: u32,
    pub state: RunningState,

    // unit: sec
    pub total_buyback_time: u32,
    // unit: sec
    pub buyback_internal: u32,
    // unit: sec
    pub current_round_start_time: u32,
    pub current_round_fund_token_id: AccountId,
    pub current_round_fund_amount: u128,
    pub current_round_fund_cost: u128,

    pub belong_burn_amount: u128,
    pub belong_company_amount: u128,
    pub belong_reward_amount: u128,

}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedContractData {
    V1000(ContractData),
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    data: VersionedContractData,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, burn_account_id: AccountId, company_account_id: AccountId, reward_account_id: AccountId, buyback_token_id: AccountId) -> Self {
        require!(!env::state_exists(), E000_ALREADY_INIT);
        Self {
            data: VersionedContractData::V1000(ContractData {
                owner_id: owner_id.clone(),
                ref_exchange_id: owner_id.clone(),
                burn_account_id, 
                company_account_id, 
                reward_account_id,
                buyback_token_id,
                token_white_list: UnorderedSet::new(StorageKeys::TokenWhiteList), 
                guardians: UnorderedSet::new(StorageKeys::Guardian), 
                
                burn_rate: 0,
                company_rate: 0,
                reward_rate: 0,
                state: RunningState::Running,

                total_buyback_time: 0,
                buyback_internal: 0,
                current_round_start_time: 0,
                current_round_fund_token_id: owner_id,
                current_round_fund_amount: 0,
                current_round_fund_cost: 0,

                belong_burn_amount: 0,
                belong_company_amount: 0,
                belong_reward_amount: 0,
            })
        }
    }
}

#[allow(unreachable_patterns)]
impl Contract {
    fn data(&self) -> &ContractData {
        match &self.data {
            VersionedContractData::V1000(data) => data,
            _ => unimplemented!(),
        }
    }

    fn data_mut(&mut self) -> &mut ContractData {
        match &mut self.data {
            VersionedContractData::V1000(data) => data,
            _ => unimplemented!(),
        }
    }

    fn assert_contract_running(&self) {
        match self.data().state {
            RunningState::Running => (),
            _ => env::panic_str(ERR6_CONTRACT_PAUSED),
        };
    }
}