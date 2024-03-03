use crate::*;
use near_sdk::{promise_result_as_success, is_promise_success};

pub const GAS_FOR_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 10);
pub const GAS_FOR_FT_TRANSFER_CALLBACK: Gas = Gas(Gas::ONE_TERA.0 * 5);
pub const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(20 * Gas::ONE_TERA.0);
pub const GAS_FOR_FT_BALANCE_OF: Gas = Gas(10 * Gas::ONE_TERA.0);
pub const GAS_FOR_FT_TRANSFER_CALL_CALLBACK: Gas = Gas(20 * Gas::ONE_TERA.0);
pub const GAS_FOR_TO_DISTRIBUTE_CALLBACK: Gas = Gas(20 * Gas::ONE_TERA.0);

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: u64,
    /// Token to swap from.
    pub token_in: AccountId,
    /// Amount to exchange.
    /// If amount_in is None, it will take amount_out from previous step.
    /// Will fail if amount_in is None on the first step.
    pub amount_in: Option<U128>,
    /// Token to swap into.
    pub token_out: AccountId,
    /// Required minimum amount of token_out.
    pub min_amount_out: U128,
}

/// Single action. Allows to execute sequence of various actions initiated by an account.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum Action {
    Swap(SwapAction),
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum SwapMessage {
    /// Alternative to deposit + execute actions call.
    Execute {
        referral_id: Option<AccountId>,
        /// List of sequential actions.
        actions: Vec<Action>,
    },
}

#[near_bindgen]
impl Contract {
    pub fn do_buyback(&mut self, swap_msg: String) {
        self.assert_contract_running();
        self.assert_owner_or_guardians();
        let swap_info = serde_json::from_str::<SwapMessage>(&swap_msg).expect(ERR100_WRONG_MSG_FORMAT);
        let available_fund_amount = self.available_fund_amount();
        let mut amount_in = 0;
        match swap_info {
            SwapMessage::Execute {
                referral_id: _,
                actions,
            } => {
                require!(!actions.is_empty(), "Invalid actions");
                for (index, action) in actions.iter().enumerate() {
                    if index == 0 {
                        match action {
                            Action::Swap(swap_action) => {
                                require!(swap_action.token_in == self.data().current_round_fund_token_id, "Invalid token_in");
                                amount_in = swap_action.amount_in.expect(ERR100_WRONG_MSG_FORMAT).0;
                                require!(amount_in > 0 && amount_in <= available_fund_amount, "Invalid amount_in");
                            } 
                        }
                    } else if index == actions.len() - 1 {
                        match action {
                            Action::Swap(swap_action) => {
                                require!(swap_action.token_out == self.data().buyback_token_id, "Invalid token_out");
                                require!(swap_action.amount_in.is_none(), "Invalid amount_in");
                            } 
                        }
                    } else {
                        match action {
                            Action::Swap(swap_action) => {
                                require!(swap_action.amount_in.is_none(), "Invalid amount_in");
                            } 
                        }
                    }
                }
            }
        }

        ext_fungible_token::ext(self.data().current_round_fund_token_id.clone())
            .with_attached_deposit(1)
            .with_static_gas(GAS_FOR_FT_TRANSFER_CALL)
            .ft_transfer_call(
                self.data().ref_exchange_id.clone(), 
                U128(amount_in), 
                None, 
                swap_msg
            ).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_FT_TRANSFER_CALL_CALLBACK)
                    .callback_do_buyback()
            );
    }

    pub fn distribute(&mut self) {
        self.assert_contract_running();
        self.assert_owner_or_guardians();
        ext_fungible_token::ext(self.data().buyback_token_id.clone())
            .with_static_gas(GAS_FOR_FT_BALANCE_OF)
            .ft_balance_of(
                env::current_account_id()
            ).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_TO_DISTRIBUTE_CALLBACK)
                    .callback_to_distribute()
            );
    }

    #[private]
    pub fn callback_do_buyback(&mut self) {
        let cross_call_result = promise_result_as_success().expect(ERR102_CROSS_CONTRACT_FAILED);
        let cost = serde_json::from_slice::<U128>(&cross_call_result).unwrap().0;
        self.data_mut().current_round_fund_cost += cost;
    }

    #[private]
    pub fn callback_to_distribute(&mut self) {
        let contract_data = self.data_mut();
        assert!(contract_data.burn_rate + contract_data.company_rate + contract_data.reward_rate == MAX_RATIO);

        let cross_call_result = promise_result_as_success().expect(ERR102_CROSS_CONTRACT_FAILED);
        let new_distrbute_amount = serde_json::from_slice::<U128>(&cross_call_result).expect(ERR102_CROSS_CONTRACT_FAILED).0
            - contract_data.belong_burn_amount
            - contract_data.belong_company_amount
            - contract_data.belong_reward_amount;

        let new_distrbute_burn_amount = ratio(new_distrbute_amount, contract_data.burn_rate);
        let new_distrbute_company_amount = ratio(new_distrbute_amount, contract_data.company_rate);
        let new_distrbute_reward_amount = new_distrbute_amount - new_distrbute_burn_amount - new_distrbute_company_amount;

        let burn_amount = new_distrbute_burn_amount + contract_data.belong_burn_amount;
        let company_amount = new_distrbute_company_amount + contract_data.belong_company_amount;
        let reward_amount = new_distrbute_reward_amount + contract_data.belong_reward_amount;
        
        contract_data.belong_burn_amount = 0;
        contract_data.belong_company_amount = 0;
        contract_data.belong_reward_amount = 0;

        if burn_amount > 0 {
            ext_fungible_token::ext(contract_data.buyback_token_id.clone())
                .with_attached_deposit(1)
                .with_static_gas(GAS_FOR_FT_TRANSFER)
                .ft_transfer(
                    contract_data.burn_account_id.clone(), 
                    U128(burn_amount), 
                    None
                ).then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_FT_TRANSFER_CALLBACK)
                        .callback_ft_transfer(contract_data.burn_account_id.clone(), U128(burn_amount))
                );
        }

        if company_amount > 0 {
            ext_fungible_token::ext(contract_data.buyback_token_id.clone())
                .with_attached_deposit(1)
                .with_static_gas(GAS_FOR_FT_TRANSFER)
                .ft_transfer(
                    contract_data.company_account_id.clone(), 
                    U128(company_amount), 
                    None
                ).then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_FT_TRANSFER_CALLBACK)
                        .callback_ft_transfer(contract_data.company_account_id.clone(), U128(company_amount))
                );
        }
        
        if reward_amount > 0 {
            ext_fungible_token::ext(contract_data.buyback_token_id.clone())
            .with_attached_deposit(1)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                contract_data.reward_account_id.clone(), 
                U128(reward_amount), 
                None
            ).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_FT_TRANSFER_CALLBACK)
                    .callback_ft_transfer(contract_data.reward_account_id.clone(), U128(reward_amount))
            );
        }
    }

    #[private]
    pub fn callback_ft_transfer(&mut self, account_id: AccountId, amount: U128) {
        if !is_promise_success() {
            if account_id == self.data().burn_account_id {
                self.data_mut().belong_burn_amount = amount.0;
            }
            if account_id == self.data().company_account_id {
                self.data_mut().belong_company_amount = amount.0;
            }
            if account_id == self.data().reward_account_id {
                self.data_mut().belong_reward_amount = amount.0;
            }
        }
    }
}

impl Contract {
    pub fn available_fund_amount(&self) -> u128 {
        let current_time = nano_to_sec(env::block_timestamp());
        let contract_data = self.data();
        if current_time <= contract_data.current_round_start_time + contract_data.total_buyback_time {
            let pass_time = current_time.checked_sub(contract_data.current_round_start_time).unwrap_or(0);
            let numerator = (pass_time / contract_data.buyback_internal) as u128;
            let denominator = (contract_data.total_buyback_time / contract_data.buyback_internal) as u128;
            contract_data.current_round_fund_amount * numerator / denominator - contract_data.current_round_fund_cost
        } else {
            contract_data.current_round_fund_amount - contract_data.current_round_fund_cost
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::VMContextBuilder;
    pub use near_sdk::{testing_env, serde_json, AccountId, Balance};

    pub fn owner_id() -> AccountId {
        AccountId::new_unchecked("owner_id".to_string())
    }
    
    pub fn burn_account_id() -> AccountId {
        AccountId::new_unchecked("burn".to_string())
    }

    pub fn company_account_id() -> AccountId {
        AccountId::new_unchecked("company".to_string())
    }

    pub fn reward_account_id() -> AccountId {
        AccountId::new_unchecked("reward".to_string())
    }

    pub fn buyback_token_id() -> AccountId {
        AccountId::new_unchecked("buyback_token".to_string())
    }

    pub fn nusdt() -> AccountId {
        AccountId::new_unchecked("nusdt".to_string())
    }

    pub fn d(value: Balance, decimals: u8) -> Balance {
        value * 10u128.pow(decimals as _)
    }

    pub fn sec_to_nano(sec: u32) -> u64 {
        u64::from(sec) * 10u64.pow(9)
    }

    #[test]
    fn base() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(owner_id()).build());
        let mut contract = Contract::new(owner_id(), burn_account_id(), company_account_id(), reward_account_id(), buyback_token_id());
        testing_env!(context.predecessor_account_id(owner_id()).attached_deposit(1).build());
        contract.extend_token_white_list(vec![nusdt()]);
        testing_env!(context.block_timestamp(sec_to_nano(1000)).predecessor_account_id(nusdt()).build());
        contract.ft_on_transfer(owner_id(), U128(d(100, 6)), serde_json::to_string(&TokenReceiverMessage::BuyBackInfo { 
            current_round_start_time: 1100, 
            total_buyback_time: 100, 
            buyback_internal: 10 
        }).unwrap());
        

        assert_eq!(contract.get_available_fund_amount().0, 0); 
        testing_env!(context.block_timestamp(sec_to_nano(1110)).build());
        assert_eq!(contract.get_available_fund_amount().0, d(10, 6)); 
        testing_env!(context.block_timestamp(sec_to_nano(1150)).build());
        assert_eq!(contract.get_available_fund_amount().0, d(50, 6)); 
        testing_env!(context.block_timestamp(sec_to_nano(1200)).build());
        assert_eq!(contract.get_available_fund_amount().0, d(100, 6)); 
    }
}