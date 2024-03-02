# buy-back

## Contract deployment

#### Private mainnet
#### ENV

```
export NEAR_ENV=mainnet
export REF_OWNER=[Owber Account]
export OWNER_ID=[Owber Id]
export MASTER=[Master Account]
export BUYBACK=[Buyback Account]
export REF_EX=v2.ref-finance.near

# distribute_rate account
export BURN_ACCOUNT=[Burn Account]
export COMPANY_ACCOUNT=[Company Account]
export REWARD_ACCOUNT=[Reward Account]

export BB_TOKEN_ACCOUNT=token.burrow.near
export BRR=token.burrow.near
export USDC=a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.factory.bridge.near

export TGAS=000000000000
export ZERO6=000000
```

### Deploy
```
near create-account $BUYBACK --masterAccount $MASTER --initialBalance 10 --accountId $MASTER
near create-account $BURN_ACCOUNT --masterAccount $MASTER --initialBalance 1 --accountId $MASTER
near create-account $COMPANY_ACCOUNT --masterAccount $MASTER --initialBalance 1 --accountId $MASTER
near create-account $REWARD_ACCOUNT --masterAccount $MASTER --initialBalance 1 --accountId $MASTER

near deploy $BUYBACK res/buyback_release.wasm --account_id=$BUYBACK || true

near call $BUYBACK new '{"owner_id": "'$REF_OWNER'", "burn_account_id": "'$BURN_ACCOUNT'", "company_account_id": "'$COMPANY_ACCOUNT'", "reward_account_id": "'$REWARD_ACCOUNT'", "buyback_token_id": "'$BB_TOKEN_ACCOUNT'"}' --account_id=$BUYBACK || true
```

### view
```
near view $BUYBACK get_metadata
near view $BUYBACK get_available_fund_amount
near view $BRR ft_balance_of '{"account_id": "'$BUYBACK'"}'
```

### Setting
```
#change ref_exchange_id 
near call $BUYBACK change_ref_exchange_id '{"ref_exchange_id": "'$REF_EX'"}' --depositYocto=1 --accountId $OWNER_ID

#add white-list
near call $BUYBACK extend_token_white_list '{"token_white_list":["'$USDC'"]}' --depositYocto=1 --accountId $OWNER_ID

#change distribute_rate
near call $BUYBACK change_buyback_rate '{"burn_rate": 2000, "company_rate": 2000, "reward_rate": 6000}' --depositYocto=1 --accountId $OWNER_ID

#register token
near view $BB_TOKEN_ACCOUNT storage_balance_of '{"account_id": "'$REWARD_ACCOUNT'"}'

near call $USDC storage_deposit '{"account_id": "'$BUYBACK'"}' --account_id=juaner.near --amount=0.1

near call $BB_TOKEN_ACCOUNT storage_deposit '{"account_id": "'$BUYBACK'"}' --account_id=juaner.near --amount=0.1

near call $BB_TOKEN_ACCOUNT storage_deposit '{"account_id": "'$BURN_ACCOUNT'"}' --account_id=juaner.near --amount=0.1

near call $BB_TOKEN_ACCOUNT storage_deposit '{"account_id": "'$COMPANY_ACCOUNT'"}' --account_id=juaner.near --amount=0.1

near call $BB_TOKEN_ACCOUNT storage_deposit '{"account_id": "'$REWARD_ACCOUNT'"}' --account_id=juaner.near --amount=0.1
```

### Testing record
```
# round 1：Open buying 1u/2h
near call $USDC ft_transfer_call '{"receiver_id": "'$BUYBACK'", "amount": "12'$ZERO6'", "msg": "{\"current_round_start_time\":1702634400,\"total_buyback_time\":86400,\"buyback_internal\":7200}"}' --accountId $OWNER_ID --depositYocto=1  --gas=300000000000000

#distribute 
near call $BUYBACK distribute --accountId $OWNER_ID --gas=300000000000000

# round 2：Open buying 1u/2h
near call $USDC ft_transfer_call '{"receiver_id": "'$BUYBACK'", "amount": "3'$ZERO6'", "msg": "{\"current_round_start_time\":1702879200,\"total_buyback_time\":21600,\"buyback_internal\":7200}"}' --accountId $OWNER_ID --depositYocto=1  --gas=300000000000000

#distribute 
near call $BUYBACK distribute --accountId $OWNER_ID --gas=300000000000000

# round 3：Open buying 1u/2h
near call $USDC ft_transfer_call '{"receiver_id": "'$BUYBACK'", "amount": "12'$ZERO6'", "msg": "{\"current_round_start_time\":1702944000,\"total_buyback_time\":86400,\"buyback_internal\":7200}"}' --accountId $OWNER_ID --depositYocto=1  --gas=300000000000000

#distribute 
near call $BUYBACK distribute --accountId $OWNER_ID --gas=300000000000000
```

## Program deployment

### Before running, it is necessary to modify the account configuration in the config file
```
private_key:Change to execute the private key of the account
signer_account_id:Change to the account that needs to be executed
buyback_contract:Modify the contract address as needed for execution
buyback_token_in_contract:Token that requires buyback
buyback_token_out_contract:Token after buyback
buyback_pool_one、buyback_pool_two:The pool involved in the token will calculate the number of swaps based on the configured token and pool
```

### Use crontab to schedule execution
#### Enter the project/backends directory and execute the following command:

```
python deploy_backend_buy_back.py
chmod a+x backend_buy_back.sh
```

#### After executing the above two commands, configure the crontab scheduled task. Example configuration (executed every 5 minutes):
```
*/5 * * * * /path/backends/backend_buy_back.sh > /dev/null
```

