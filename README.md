# buy-back

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

