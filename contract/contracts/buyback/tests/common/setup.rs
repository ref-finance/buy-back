use crate::*;

pub const PREVIOUS_BUYBACK_WASM: &str = "../../res/buyback.wasm";
pub const BUYBACK_WASM: &str = "../../res/buyback.wasm";
const REF_EXCHANGE_WASM: &str = "../../res/mock_ref_exchange.wasm";
const FT_WASM: &str = "../../res/mock_ft.wasm";

pub async fn deploy_buyback(
    root: &Account,
    owner_id: &Account, 
    burn_account: &Account, 
    company_account: &Account, 
    reward_account: &Account,
    buyback_token: &Account
) -> Result<BuyBackContract> {
    let buyback = root
        .create_subaccount("buyback")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let buyback = buyback
        .deploy(&std::fs::read(BUYBACK_WASM).unwrap())
        .await?
        .unwrap();
    assert!(buyback.call("new")
        .args_json(json!({
            "owner_id": owner_id.id(), 
            "burn_account_id": burn_account.id(), 
            "company_account_id": company_account.id(), 
            "reward_account_id": reward_account.id(),
            "buyback_token_id": buyback_token.id()
        }))
        .max_gas()
        .transact()
        .await?
        .is_success());
    Ok(BuyBackContract(buyback))
}

pub async fn deploy_previous_version_buyback(
    root: &Account,
    owner_id: &Account, 
    burn_account: &Account, 
    company_account: &Account, 
    reward_account: &Account,
    buyback_token: &Account
) -> Result<BuyBackContract> {
    let buyback = root
        .create_subaccount("buyback")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let buyback = buyback
        .deploy(&std::fs::read(PREVIOUS_BUYBACK_WASM).unwrap())
        .await?
        .unwrap();
    assert!(buyback.call("new")
        .args_json(json!({
            "owner_id": owner_id.id(), 
            "burn_account_id": burn_account.id(), 
            "company_account_id": company_account.id(), 
            "reward_account_id": reward_account.id(),
            "buyback_token_id": buyback_token.id()
        }))
        .max_gas()
        .transact()
        .await?
        .is_success());
    Ok(BuyBackContract(buyback))
}

pub async fn deploy_mock_ft(
    root: &Account,
    symbol: &str,
    decimal: u8,
) -> Result<FtContract> {

    let mock_ft = root
        .create_subaccount(symbol)
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let mock_ft = mock_ft
        .deploy(&std::fs::read(FT_WASM).unwrap())
        .await?
        .unwrap();
    assert!(mock_ft
        .call("new")
        .args_json(json!({
            "name": symbol,
            "symbol": symbol,
            "decimals": decimal,
        }))
        .gas(300_000_000_000_000)
        .transact()
        .await?
        .is_success());
    Ok(FtContract(mock_ft))
}

pub async fn deploy_ref_exchange(
    root: &Account,
) -> Result<RefExchange> {
    let ref_exchange = root
        .create_subaccount("ref_exchange")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let ref_exchange = ref_exchange
        .deploy(&std::fs::read(REF_EXCHANGE_WASM).unwrap())
        .await?
        .unwrap();
    assert!(ref_exchange.call("new")
        .args_json(json!({
            "owner_id": root.id(),
            "exchange_fee": 2000,
            "referral_fee": 0,
            "boost_farm_id": near_sdk::AccountId::new_unchecked("boost_farming.test.near".to_string()),
            "burrowland_id": near_sdk::AccountId::new_unchecked("burrowland.test.near".to_string()),
        }))
        .max_gas()
        .transact()
        .await?
        .is_success());
    Ok(RefExchange(ref_exchange))
}