
use crate::*;

pub struct BuyBackContract(pub Contract);

impl BuyBackContract {
    pub async fn extend_guardians(
        &self,
        caller: &Account,
        guardians: Vec<&AccountId>
    ) -> Result<ExecutionFinalResult> {
        caller
            .call(self.0.id(), "extend_guardians")
            .args_json(json!({
                "guardians": guardians,
            }))
            .gas(20_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn extend_token_white_list(
        &self,
        caller: &Account,
        token_white_list: Vec<&AccountId>
    ) -> Result<ExecutionFinalResult> {
        caller
            .call(self.0.id(), "extend_token_white_list")
            .args_json(json!({
                "token_white_list": token_white_list,
            }))
            .gas(20_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn change_ref_exchange_id(
        &self,
        caller: &Account,
        ref_exchange_id: &AccountId
    ) -> Result<ExecutionFinalResult> {
        caller
            .call(self.0.id(), "change_ref_exchange_id")
            .args_json(json!({
                "ref_exchange_id": ref_exchange_id,
            }))
            .gas(20_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn init_buyback_round(
        &self,
        token_contract: &FtContract,
        caller: &Account,
        amount: u128,
        msg: String
    ) -> Result<ExecutionFinalResult> {
        token_contract.ft_transfer_call(caller, self.0.id(), amount, msg).await
    }

    pub async fn do_buyback(
        &self,
        caller: &Account,
        swap_msg: String
    ) -> Result<ExecutionFinalResult> {
        caller
            .call(self.0.id(), "do_buyback")
            .args_json(json!({
                "swap_msg": swap_msg,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn distribute(
        &self,
        caller: &Account,
    ) -> Result<ExecutionFinalResult> {
        caller
            .call(self.0.id(), "distribute")
            .max_gas()
            .transact()
            .await
    }
    
    pub async fn change_buyback_rate(
        &self,
        caller: &Account,
        burn_rate: u32, 
        company_rate: u32, 
        reward_rate: u32
    ) -> Result<ExecutionFinalResult> {
        caller
            .call(self.0.id(), "change_buyback_rate")
            .args_json(json!({
                "burn_rate": burn_rate, 
                "company_rate": company_rate, 
                "reward_rate": reward_rate
            }))
            .max_gas()
            .deposit(1)
            .transact()
            .await
    }
}

impl BuyBackContract {
    pub async fn get_metadata(
        &self,
    ) -> Result<ContractMetadata> {
        self.0
            .call("get_metadata")
            .view()
            .await?
            .json::<ContractMetadata>()
    }

    pub async fn get_available_fund_amount(
        &self,
    ) -> Result<U128> {
        self.0
            .call("get_available_fund_amount")
            // .args_json(json!({
            // }))
            .view()
            .await?
            .json::<U128>()
    }
    
}