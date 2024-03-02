# buy-back

### 运行前需要修改config文件中的账号配置
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

