use crate::*;

impl Contract {
    pub fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.data().owner_id,
            E002_NOT_ALLOWED
        );
    }

    pub(crate) fn assert_owner_or_guardians(&self) {
        require!(env::predecessor_account_id() == self.data().owner_id
            || self.data().guardians.contains(&env::predecessor_account_id()), 
            E002_NOT_ALLOWED)
    }
}

#[near_bindgen]
impl Contract {
    /// Change owner. Only can be called by owner.
    #[payable]
    pub fn set_owner(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.data_mut().owner_id = owner_id;
    }

    /// Extend guardians. Only can be called by owner.
    #[payable]
    pub fn extend_guardians(&mut self, guardians: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner();
        for guardian in guardians {
            self.data_mut().guardians.insert(&guardian);
        }
    }

    /// Remove guardians. Only can be called by owner.
    #[payable]
    pub fn remove_guardians(&mut self, guardians: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner();
        for guardian in guardians {
            let is_success = self.data_mut().guardians.remove(&guardian);
            require!(is_success, E004_INVALID_GUARDIAN);
        }
    }

    /// Extend guardians. Only can be called by owner.
    #[payable]
    pub fn extend_token_white_list(&mut self, token_white_list: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner_or_guardians();
        for token in token_white_list {
            self.data_mut().token_white_list.insert(&token);
        }
    }

    /// Remove guardians. Only can be called by owner.
    #[payable]
    pub fn remove_token_white_list(&mut self, token_white_list: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner_or_guardians();
        for token in token_white_list {
            let is_success = self.data_mut().token_white_list.remove(&token);
            require!(is_success, E005_INVALID_TOKEN);
        }
    }

    #[payable]
    pub fn change_state(&mut self, state: RunningState) {
        assert_one_yocto();
        self.assert_owner_or_guardians();

        if self.data().state != state {
            if state == RunningState::Running {
                // only owner can resume the contract
                self.assert_owner();
            }
            log!("{}",
                format!(
                    "Contract state changed from {} to {} by {}",
                    self.data().state, state, env::predecessor_account_id()
                )
                
            );     
            self.data_mut().state = state;
        }
    }

    #[payable]
    pub fn change_buyback_rate(&mut self, burn_rate: u32, company_rate: u32, reward_rate: u32) {
        assert_one_yocto();
        self.assert_owner_or_guardians();

        assert!(burn_rate + company_rate + reward_rate == MAX_RATIO);
        self.data_mut().burn_rate = burn_rate;
        self.data_mut().company_rate = company_rate;
        self.data_mut().reward_rate = reward_rate;
    }

    #[payable]
    pub fn change_burn_account_id(&mut self, burn_account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner_or_guardians();
        require!(self.data().belong_burn_amount == 0);
        self.data_mut().burn_account_id = burn_account_id;
    }

    #[payable]
    pub fn change_company_account_id(&mut self, company_account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner_or_guardians();
        require!(self.data().belong_company_amount == 0);
        self.data_mut().company_account_id = company_account_id;
    }

    #[payable]
    pub fn change_reward_account_id(&mut self, reward_account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner_or_guardians();
        require!(self.data().belong_reward_amount == 0);
        self.data_mut().reward_account_id = reward_account_id;
    }

    #[payable]
    pub fn change_buyback_token_id(&mut self, buyback_token_id: AccountId) {
        assert_one_yocto();
        self.assert_owner_or_guardians();
        require!(self.data().belong_burn_amount == 0 
            && self.data().belong_company_amount == 0
            && self.data().belong_reward_amount == 0);
        self.data_mut().buyback_token_id = buyback_token_id;
    }

    #[payable]
    pub fn change_ref_exchange_id(&mut self, ref_exchange_id: AccountId) {
        assert_one_yocto();
        self.assert_owner_or_guardians();
        self.data_mut().ref_exchange_id = ref_exchange_id;
    }
}

/// Upgrade ralated
#[near_bindgen]
impl Contract {
    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have, you need to implement migration from old state
    /// (keep the old struct with different name to deserialize it first).
    /// After migration goes live, revert back to this implementation for next updates.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let mut contract: Contract = env::state_read().expect(E003_NOT_INIT);
        // see if ContractData need upgrade
        contract.data = match contract.data {
            VersionedContractData::V1000(data) => VersionedContractData::V1000(data),
        };
        contract
    }
}

mod upgrade {
    use near_sdk::{require, Gas};
    use near_sys as sys;

    use super::*;

    const GAS_TO_COMPLETE_UPGRADE_CALL: Gas = Gas(Gas::ONE_TERA.0 * 10);
    const GAS_FOR_GET_CONFIG_CALL: Gas = Gas(Gas::ONE_TERA.0 * 5);
    const MIN_GAS_FOR_MIGRATE_STATE_CALL: Gas = Gas(Gas::ONE_TERA.0 * 60);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub extern "C" fn upgrade() {
        env::setup_panic_hook();
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_account_id = env::current_account_id().as_bytes().to_vec();
        let migrate_method_name = b"migrate".to_vec();
        let get_metadata_method_name = b"get_metadata".to_vec();
        let empty_args = b"{}".to_vec();
        unsafe {
            // Load input (wasm code) into register 0.
            sys::input(0);
            // Create batch action promise for the current contract ID
            let promise_id = sys::promise_batch_create(
                current_account_id.len() as _,
                current_account_id.as_ptr() as _,
            );
            // 1st action in the Tx: "deploy contract" (code is taken from register 0)
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // Gas required to complete this call.
            let required_gas =
                env::used_gas() + GAS_TO_COMPLETE_UPGRADE_CALL + GAS_FOR_GET_CONFIG_CALL;
            require!(
                env::prepaid_gas() >= required_gas + MIN_GAS_FOR_MIGRATE_STATE_CALL,
                "Not enough gas to complete state migration"
            );
            let migrate_state_attached_gas = env::prepaid_gas() - required_gas;
            // 2nd action in the Tx: call this_contract.migrate() with remaining gas
            sys::promise_batch_action_function_call(
                promise_id,
                migrate_method_name.len() as _,
                migrate_method_name.as_ptr() as _,
                empty_args.len() as _,
                empty_args.as_ptr() as _,
                0 as _,
                migrate_state_attached_gas.0,
            );
            // Scheduling to return config after the migration is completed.
            //
            // The upgrade method attaches it as an action, so the entire upgrade including deploy
            // contract action and migration can be rolled back if the config view call can't be
            // returned successfully. The view call deserializes the state and deserializes the
            // config which contains the owner_id. If the contract can deserialize the current config,
            // then it can validate the owner and execute the upgrade again (in case the previous
            // upgrade/migration went badly).
            //
            // It's an extra safety guard for the remote contract upgrades.
            sys::promise_batch_action_function_call(
                promise_id,
                get_metadata_method_name.len() as _,
                get_metadata_method_name.as_ptr() as _,
                empty_args.len() as _,
                empty_args.as_ptr() as _,
                0 as _,
                GAS_FOR_GET_CONFIG_CALL.0,
            );
            sys::promise_return(promise_id);
        }
    }
}