use crate::*;

pub async fn create_account(
    master: &Account,
    account_id: &str,
    balance: Option<u128>,
) -> Account {
    let balance = if let Some(balance) = balance {
        balance
    } else {
        parse_near!("50 N")
    };
    master
        .create_subaccount(account_id)
        .initial_balance(balance)
        .transact()
        .await
        .unwrap()
        .unwrap()
}

pub fn tool_err_msg(outcome: Result<ExecutionFinalResult>) -> String {
    match outcome {
        Ok(res) => {
            let mut msg = "".to_string();
            for r in res.receipt_failures(){
                match r.clone().into_result() {
                    Ok(_) => {},
                    Err(err) => {
                        msg += &format!("{:?}", err);
                        msg += "\n";
                    }
                }
            }
            msg
        },
        Err(err) => err.to_string()
    }
}

#[macro_export]
macro_rules! check{
    ($exec_func: expr)=>{
        let outcome = $exec_func.await?;
        assert!(outcome.is_success() && outcome.receipt_failures().is_empty());
    };
    (print $exec_func: expr)=>{
        let outcome = $exec_func.await;
        let err_msg = tool_err_msg(outcome);
        if err_msg.is_empty() {
            println!("success");
        } else {
            println!("{}", err_msg);
        }
    };
    (print $prefix: literal $exec_func: expr)=>{
        let outcome = $exec_func.await;
        let err_msg = tool_err_msg(outcome);
        if err_msg.is_empty() {
            println!("{} success", $prefix);
        } else {
            println!("{} {}", $prefix, err_msg);
        }
    };
    (view $exec_func: expr)=>{
        let query_result = $exec_func.await?;
        println!("{:?}", query_result);
    };
    (view $prefix: literal $exec_func: expr)=>{
        let query_result = $exec_func.await?;
        println!("{} {:?}", $prefix, query_result);
    };
    (logs $exec_func: expr)=>{
        let outcome = $exec_func.await?;
        assert!(outcome.is_success() && outcome.receipt_failures().is_empty());
        println!("{:#?}", outcome.logs());
    };
    ($exec_func: expr, $err_info: expr)=>{
        assert!(tool_err_msg($exec_func.await).contains($err_info));
    };
}