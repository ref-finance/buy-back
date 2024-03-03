mod common;

use crate::common::*;

const PREVIOUS_VERSION: &'static str = "0.1.0";
const LATEST_VERSION: &'static str = "0.1.0";

#[tokio::test]
async fn test_upgrade() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let root = worker.root_account()?;

    let owner = create_account(&root, "owner", None).await;
    let burn = create_account(&root, "burn", None).await;
    let company = create_account(&root, "company", None).await;
    let reward = create_account(&root, "reward", None).await;

    let brrr_token_contract = deploy_mock_ft(&root, "brrr", 18).await?;

    let previous_burrowland_contract = deploy_previous_version_buyback(&root, &owner, &burn, &company, &reward, brrr_token_contract.0.as_account()).await?;
    let metadata = previous_burrowland_contract.get_metadata().await?;
    assert_eq!(metadata.version, PREVIOUS_VERSION);

    assert!(owner
        .call(previous_burrowland_contract.0.id(), "upgrade")
        .args(std::fs::read(BUYBACK_WASM).unwrap())
        .max_gas()
        .transact()
        .await?.is_success());
    let metadata = previous_burrowland_contract.get_metadata().await?;
    assert_eq!(metadata.version, LATEST_VERSION);
    Ok(())
}
