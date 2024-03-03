mod common;

use crate::common::*;

#[tokio::test]
async fn test_base() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let root = worker.root_account()?;

    let owner = create_account(&root, "owner", None).await;
    let guardian = create_account(&root, "guardian", None).await;
    let burn = create_account(&root, "burn", None).await;
    let company = create_account(&root, "company", None).await;
    let reward = create_account(&root, "reward", None).await;

    let usdt_token_contract = deploy_mock_ft(&root, "nusdt", 6).await?;
    let usdc_token_contract = deploy_mock_ft(&root, "nusdc", 6).await?;
    let brrr_token_contract = deploy_mock_ft(&root, "brrr", 18).await?;
    {
        // check!(brrr_token_contract.ft_storage_deposit(company.id()));
        // check!(brrr_token_contract.ft_storage_deposit(reward.id()));
    }

    let buyback_contract = deploy_buyback(&root, &owner, &burn, &company, &reward, brrr_token_contract.0.as_account()).await?;
    {
        check!(usdt_token_contract.ft_storage_deposit(buyback_contract.0.id()));
        check!(usdc_token_contract.ft_storage_deposit(buyback_contract.0.id()));
        check!(brrr_token_contract.ft_storage_deposit(buyback_contract.0.id()));
        check!(buyback_contract.extend_guardians(&owner, vec![guardian.id()]));
        check!(buyback_contract.extend_token_white_list(&owner, vec![usdt_token_contract.0.id(), usdc_token_contract.0.id()]));
        check!(view buyback_contract.get_metadata());
    }

    let ref_exchange_contract = deploy_ref_exchange(&root).await?;
    {
        check!(buyback_contract.change_ref_exchange_id(&owner, ref_exchange_contract.0.id()));
        check!(usdt_token_contract.ft_storage_deposit(ref_exchange_contract.0.id()));
        check!(usdc_token_contract.ft_storage_deposit(ref_exchange_contract.0.id()));
        check!(brrr_token_contract.ft_storage_deposit(ref_exchange_contract.0.id()));
        check!(ref_exchange_contract.storage_deposit(&root));
        check!(ref_exchange_contract.extend_whitelisted_tokens(&root, vec![usdt_token_contract.0.id(), usdc_token_contract.0.id(), brrr_token_contract.0.id()]));
    }

    let alice = create_account(&root, "alice", None).await;
    {
        check!(ref_exchange_contract.storage_deposit(&alice));
        assert!(usdt_token_contract.ft_mint(&root, &alice, 10000 * 10u128.pow(6)).await?.is_success());
        assert!(brrr_token_contract.ft_mint(&root, &alice, 10000 * 10u128.pow(18)).await?.is_success());
        check!(ref_exchange_contract.add_simple_pool(&root, vec![usdt_token_contract.0.id(), brrr_token_contract.0.id()], 5));
        check!(ref_exchange_contract.deposit(&usdt_token_contract, &alice, 10000 * 10u128.pow(6)));
        check!(ref_exchange_contract.deposit(&brrr_token_contract, &alice, 10000 * 10u128.pow(18)));
        check!(ref_exchange_contract.add_liquidity(&alice, 0, vec![U128(10000 * 10u128.pow(6)), U128(10000 * 10u128.pow(18))], None));
        check!(view ref_exchange_contract.get_pool(0));
    }
    
    assert!(usdt_token_contract.ft_mint(&root, &owner, 10000 * 10u128.pow(6)).await?.is_success());

    let current_timestamp = nano_to_sec(worker.view_block().await?.timestamp());
    let msg = serde_json::to_string(&TokenReceiverMessage::BuyBackInfo {
        current_round_start_time: current_timestamp,
        total_buyback_time: 100,
        buyback_internal: 10
    }).unwrap();
    check!(buyback_contract.init_buyback_round(&usdt_token_contract, &owner, 10000 * 10u128.pow(6), msg));
    let mut available_fund_amount = 0;
    while available_fund_amount == 0 {
        worker.fast_forward(10).await?;
        available_fund_amount = buyback_contract.get_available_fund_amount().await?.0;
    }
    println!("{:?}", available_fund_amount);

    let swap_msg = serde_json::to_string(&SwapMessage::Execute {
        referral_id: None,
        actions: vec![
            Action::Swap (
                SwapAction {
                    pool_id: 0,
                    token_in: near_sdk::AccountId::new_unchecked(usdt_token_contract.0.id().to_string()),
                    amount_in: Some(U128(available_fund_amount)),
                    token_out: near_sdk::AccountId::new_unchecked(brrr_token_contract.0.id().to_string()),
                    min_amount_out: U128(0),
                }
            )
        ]
    }).unwrap();
    check!(view ref_exchange_contract.get_return(0, usdt_token_contract.0.id(), available_fund_amount, brrr_token_contract.0.id()));
    check!(view "brrr buyback_contract balance" brrr_token_contract.ft_balance_of(&buyback_contract.0.as_account()));
    check!(view "usdt buyback_contract balance" usdt_token_contract.ft_balance_of(&buyback_contract.0.as_account()));
    check!(logs buyback_contract.do_buyback(&guardian, swap_msg));
    check!(view "usdt buyback_contract balance" usdt_token_contract.ft_balance_of(&buyback_contract.0.as_account()));
    check!(view "brrr buyback_contract balance" brrr_token_contract.ft_balance_of(&buyback_contract.0.as_account()));

    check!(view buyback_contract.get_metadata());

    check!(view "brrr burn balance" brrr_token_contract.ft_balance_of(&burn));
    check!(view "brrr company balance" brrr_token_contract.ft_balance_of(&company));
    check!(view "brrr reward alance" brrr_token_contract.ft_balance_of(&reward));
    check!(buyback_contract.change_buyback_rate(&guardian, 2000, 2000, 6000));
    check!(print buyback_contract.distribute(&guardian));
    check!(view "brrr burn balance" brrr_token_contract.ft_balance_of(&burn));
    check!(view "brrr company balance" brrr_token_contract.ft_balance_of(&company));
    check!(view "brrr reward alance" brrr_token_contract.ft_balance_of(&reward));
    check!(view "brrr buyback_contract balance" brrr_token_contract.ft_balance_of(&buyback_contract.0.as_account()));

    // check!(view buyback_contract.get_metadata());

    // check!(brrr_token_contract.ft_storage_deposit(burn.id()));
    // check!(print buyback_contract.distribute(&guardian));
    // check!(view "brrr burn balance" brrr_token_contract.ft_balance_of(&burn));
    // check!(view "brrr company balance" brrr_token_contract.ft_balance_of(&company));
    // check!(view "brrr reward alance" brrr_token_contract.ft_balance_of(&reward));
    // check!(view "brrr buyback_contract balance" brrr_token_contract.ft_balance_of(&buyback_contract.0.as_account()));

    // check!(view buyback_contract.get_metadata());

    available_fund_amount = 0;
    while available_fund_amount == 0 {
        worker.fast_forward(10).await?;
        available_fund_amount = buyback_contract.get_available_fund_amount().await?.0;
    }

    let swap_msg = serde_json::to_string(&SwapMessage::Execute {
        referral_id: None,
        actions: vec![
            Action::Swap (
                SwapAction {
                    pool_id: 0,
                    token_in: near_sdk::AccountId::new_unchecked(usdt_token_contract.0.id().to_string()),
                    amount_in: Some(U128(available_fund_amount)),
                    token_out: near_sdk::AccountId::new_unchecked(brrr_token_contract.0.id().to_string()),
                    min_amount_out: U128(0),
                }
            )
        ]
    }).unwrap();

    check!(view ref_exchange_contract.get_return(0, usdt_token_contract.0.id(), available_fund_amount, brrr_token_contract.0.id()));
    check!(logs buyback_contract.do_buyback(&guardian, swap_msg));
    check!(print buyback_contract.distribute(&guardian));

    check!(view "brrr buyback_contract balance" brrr_token_contract.ft_balance_of(&buyback_contract.0.as_account()));
    check!(view buyback_contract.get_metadata());

    Ok(())
}


