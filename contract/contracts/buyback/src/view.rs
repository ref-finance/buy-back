use crate::*;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Debug))]
pub struct ContractMetadata {
    pub version: String,
    pub owner: AccountId,
    pub ref_exchange_id: AccountId,
    pub burn_account_id: AccountId, 
    pub company_account_id: AccountId, 
    pub reward_account_id: AccountId,
    pub buyback_token_id: AccountId,
    pub token_white_list: Vec<AccountId>,
    pub guardians: Vec<AccountId>,
    pub burn_rate: u32,
    pub company_rate: u32,
    pub reward_rate: u32,
    pub state: RunningState,

    pub total_buyback_time: u32,
    pub buyback_internal: u32,
    pub current_round_start_time: u32,
    pub current_round_fund_token_id: AccountId,
    pub current_round_fund_amount: U128,
    pub current_round_fund_cost: U128,

    pub belong_burn_amount: U128,
    pub belong_company_amount: U128,
    pub belong_reward_amount: U128,
}

#[near_bindgen]
impl Contract {

    /// Return contract basic info
    pub fn get_metadata(&self) -> ContractMetadata {
        let contract_data = self.data();
        ContractMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner: contract_data.owner_id.clone(),
            ref_exchange_id: contract_data.ref_exchange_id.clone(),
            burn_account_id: contract_data.burn_account_id.clone(),
            company_account_id: contract_data.company_account_id.clone(),
            reward_account_id: contract_data.reward_account_id.clone(),
            buyback_token_id: contract_data.buyback_token_id.clone(),
            token_white_list: contract_data.token_white_list.to_vec(),
            guardians: contract_data.guardians.to_vec(),
            burn_rate: contract_data.burn_rate,
            company_rate: contract_data.company_rate,
            reward_rate: contract_data.reward_rate,
            state: contract_data.state.clone(),

            total_buyback_time: contract_data.total_buyback_time,
            buyback_internal: contract_data.buyback_internal,
            current_round_start_time: contract_data.current_round_start_time,
            current_round_fund_token_id: contract_data.current_round_fund_token_id.clone(),
            current_round_fund_amount: U128(contract_data.current_round_fund_amount),
            current_round_fund_cost: U128(contract_data.current_round_fund_cost),

            belong_burn_amount: U128(contract_data.belong_burn_amount),
            belong_company_amount: U128(contract_data.belong_company_amount),
            belong_reward_amount: U128(contract_data.belong_reward_amount),
        }
    }

    pub fn get_available_fund_amount(&self) -> U128 {
        U128(self.available_fund_amount())
    }
}