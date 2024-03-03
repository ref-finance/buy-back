use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum TokenReceiverMessage {
    BuyBackInfo {
        current_round_start_time: u32,
        total_buyback_time: u32,
        buyback_internal: u32,
    },
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is either "" for deposit or `TokenReceiverMessage`.
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.assert_contract_running();
        require!(sender_id == self.data().owner_id ||
            self.data().guardians.contains(&sender_id), E002_NOT_ALLOWED);
        let token_id = env::predecessor_account_id();

        if self.data().current_round_fund_amount != self.data().current_round_fund_cost {
            env::panic_str(ERR101_BUYBACK_IN_PROGRESS);
        }

        let message =
                serde_json::from_str::<TokenReceiverMessage>(&msg).expect(ERR100_WRONG_MSG_FORMAT);
        match message {
            TokenReceiverMessage::BuyBackInfo { current_round_start_time, total_buyback_time, buyback_internal } => {
                require!(self.data().token_white_list.contains(&token_id), "Invalid token_id");

                let contract_data = self.data_mut();
                contract_data.total_buyback_time = total_buyback_time;
                contract_data.buyback_internal = buyback_internal;

                contract_data.current_round_start_time = current_round_start_time;
                contract_data.current_round_fund_token_id = token_id;
                contract_data.current_round_fund_amount = amount.0;
                contract_data.current_round_fund_cost = 0;
            }
        }

        PromiseOrValue::Value(U128(0))
    }
}