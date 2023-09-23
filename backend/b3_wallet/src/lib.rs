mod guard;

use crate::guard::{caller_is_admin, caller_is_canister_or_admin, caller_is_signer};
use b3_operations::{
    error::OperationError,
    operation::{
        btc::transfer::BtcTransfer,
        global::SendToken,
        icp::transfer::IcpTransfer,
        inner::account::{CreateAccount, RemoveAccount, RenameAccount},
        inner::setting::{UpdateCanisterSettings, UpgradeCanister},
        inner::user::AddUser,
        Operation, OperationState, OperationTrait,
    },
    pending::RequestArgs,
    processed::ProcessedOperation,
    response::Response,
    role::{AccessLevel, Role},
    store::{
        with_operation, with_operation_mut, with_pending_operation_mut, with_processed_operation,
        with_processed_operation_mut, with_user, with_users, with_users_mut,
        with_users_who_can_operate, with_verified_user,
    },
    types::{PendingOperations, ProcessedOperations, UserMap, WalletSettingsAndSigners},
    user::User,
};
use b3_utils::{
    ic_canister_status,
    ledger::{
        currency::{ICPToken, TokenAmount},
        types::{
            Cycles, NotifyTopUpResult, TransferBlockIndex, WalletAccountsNonce,
            WalletCanisterInitArgs, WalletCanisterStatus, WalletController, WalletControllerMap,
            WalletInititializeArgs,
        },
    },
    revert,
    types::{CanisterId, ControllerId, Metadata, OperationId, UserId},
    wasm::{with_wasm, with_wasm_mut, WasmDetails, WasmHash, WasmSize},
    Environment, NanoTimeStamp, Subaccount,
};
use b3_wallet_lib::{
    account::WalletAccount,
    error::WalletError,
    ledger::{
        btc::network::BtcNetwork,
        chain::ChainTrait,
        ckbtc::{
            minter::Minter,
            types::{RetrieveBtcStatus, UtxoStatus},
        },
        subaccount::SubaccountEcdsaTrait,
        types::{AddressMap, Balance, BtcPending, ChainEnum, PendingEnum, SendResult},
    },
    setting::WalletSettings,
    state::WalletState,
    store::{
        with_account, with_account_mut, with_chain, with_chain_mut, with_ledger, with_ledger_mut,
        with_setting, with_setting_mut, with_wallet, with_wallet_mut,
    },
    types::{AccountId, WalletAccountView},
};
use ic_cdk::{
    api::{
        call::arg_data,
        management_canister::{
            bitcoin::Satoshi,
            main::{install_code, uninstall_code, CanisterInstallMode, InstallCodeArgument},
            provisional::CanisterIdRecord,
        },
    },
    init, post_upgrade, pre_upgrade, query, update,
};

#[init]
pub fn init() {
    // when the canister is created by another canister (e.g. the system canister)
    // this function is called with the arguments passed to the canister constructor.
    let (call_arg,) = arg_data::<(Option<WalletCanisterInitArgs>,)>();

    let mut signers = UserMap::new();

    let owner_id = match call_arg {
        Some(WalletCanisterInitArgs {
            owner_id,
            system_id,
        }) => {
            let role = Role::new("system".to_owned(), AccessLevel::ReadOnly);
            // if the canister is created by the system canister, the system canister
            // is added as trusted Canister
            signers.insert(system_id, User::new(role, "System".to_owned(), None));
            owner_id
        }
        None => ic_cdk::caller(),
    };

    let role = Role::new("owner".to_owned(), AccessLevel::FullAccess);

    signers.insert(owner_id, User::new(role, "Owner".to_owned(), None));

    with_users_mut(|users| users.set_users(signers));

    // set initial controllers
    with_setting_mut(|s| {
        s.controllers
            .insert(ic_cdk::id(), WalletController::new("Self".to_owned(), None));

        s.controllers
            .insert(owner_id, WalletController::new("Owner".to_owned(), None));
    });
}

#[pre_upgrade]
pub fn pre_upgrade() {
    with_wasm_mut(|wasm| wasm.unload());

    let permit = with_operation(|o| o.clone());
    let state = with_wallet(|s| s.clone());

    ic_cdk::storage::stable_save((state, permit)).unwrap();
}

#[post_upgrade]
pub fn post_upgrade() {
    let (state_prev, sign_prev): (WalletState, OperationState) =
        ic_cdk::storage::stable_restore().unwrap();

    with_wallet_mut(|state| *state = state_prev);

    with_operation_mut(|permit| *permit = sign_prev);
}

#[query(guard = "caller_is_signer")]
pub fn get_account(account_id: AccountId) -> WalletAccount {
    with_account(&account_id, |account| account.clone()).unwrap_or_else(revert)
}

#[query(guard = "caller_is_signer")]
pub fn get_account_count() -> usize {
    with_wallet(|s| s.accounts_len())
}

#[query(guard = "caller_is_signer")]
pub fn get_account_counters() -> WalletAccountsNonce {
    with_wallet(|s| s.counters().clone())
}

#[query(guard = "caller_is_signer")]
pub fn get_account_views() -> Vec<WalletAccountView> {
    with_wallet(|s| s.account_views())
}

#[query(guard = "caller_is_signer")]
pub fn get_account_view(account_id: AccountId) -> WalletAccountView {
    with_account(&account_id, |account| account.view()).unwrap_or_else(revert)
}

#[query(guard = "caller_is_signer")]
pub fn get_addresses(account_id: AccountId) -> AddressMap {
    with_ledger(&account_id, |ledger| ledger.address_map().clone()).unwrap_or_else(revert)
}

#[query(guard = "caller_is_signer")]
pub async fn retrieve_btc_status(
    network: BtcNetwork,
    block_index: TransferBlockIndex,
) -> RetrieveBtcStatus {
    let minter = Minter(network);

    minter
        .retrieve_btc_status(block_index)
        .await
        .unwrap_or_else(revert)
}

// UPDATE ---------------------------------------------------------------------
#[update(guard = "caller_is_signer")]
pub async fn account_update_balance(account_id: AccountId, network: BtcNetwork) -> Vec<UtxoStatus> {
    let ckbtc = with_chain(&account_id, &ChainEnum::CKBTC(network), |chain| {
        chain.ckbtc()
    })
    .unwrap_or_else(revert)
    .unwrap_or_else(revert);

    let reuslt = ckbtc.update_balance().await.unwrap_or_else(revert);

    match reuslt {
        Ok(utxos) => utxos,
        Err(err) => revert(err),
    }
}

#[update(guard = "caller_is_signer")]
pub fn account_create(env: Option<Environment>, name: Option<String>) {
    let subaccount = with_wallet(|s| s.new_subaccount(env));

    let new_account = WalletAccount::from(subaccount);

    with_wallet_mut(|s| s.insert_account(new_account, name));
}

#[update(guard = "caller_is_signer")]
pub fn account_rename(account_id: AccountId, name: String) {
    with_account_mut(&account_id, |a| a.rename(name)).unwrap_or_else(revert)
}

#[update(guard = "caller_is_signer")]
pub fn account_hide(account_id: AccountId) {
    with_account_mut(&account_id, |a| a.hide()).unwrap_or_else(revert)
}

#[update(guard = "caller_is_signer")]
pub fn account_remove(account_id: AccountId) {
    with_wallet_mut(|s| s.remove_account(&account_id)).unwrap_or_else(revert);
}

#[update(guard = "caller_is_signer")]
pub fn account_remove_address(account_id: AccountId, chain: ChainEnum) {
    with_ledger_mut(&account_id, |ledger| ledger.remove_address(chain))
        .unwrap_or_else(revert)
        .unwrap_or_else(revert);
}

#[update(guard = "caller_is_signer")]
pub fn account_restore(env: Environment, nonce: u64) {
    let subaccount = Subaccount::new(env, nonce);

    with_wallet_mut(|s| s.restore_account(subaccount)).unwrap_or_else(revert);
}

#[update(guard = "caller_is_signer")]
pub async fn account_balance(account_id: AccountId, chain: ChainEnum) -> Balance {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let balance = ledger.balance(chain).await;

    match balance {
        Ok(balance) => balance,
        Err(err) => revert(err),
    }
}

#[update(guard = "caller_is_signer")]
pub async fn account_send(
    account_id: AccountId,
    chain: ChainEnum,
    to: String,
    amount: TokenAmount,
) -> SendResult {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    ledger.send(&chain, to, amount).await.unwrap_or_else(revert)
}

#[update(guard = "caller_is_signer")]
pub async fn account_check_pending(
    account_id: AccountId,
    chain_enum: ChainEnum,
    pending_index: usize,
) {
    let chain = with_chain(&account_id, &chain_enum, |chain| chain.clone()).unwrap_or_else(revert);

    let result = chain.check_pending(pending_index).await;

    match result {
        Ok(_) => {
            with_chain_mut(&account_id, chain_enum, |chain| {
                chain.remove_pending(pending_index)
            })
            .unwrap_or_else(revert);
        }
        Err(err) => revert(err),
    }
}

#[update(guard = "caller_is_signer")]
pub async fn account_add_pending(account_id: AccountId, chain: ChainEnum, pending: PendingEnum) {
    with_chain_mut(&account_id, chain, |chain| chain.add_pending(pending)).unwrap_or_else(revert);
}

#[update(guard = "caller_is_signer")]
pub async fn account_remove_pending(account_id: AccountId, chain: ChainEnum, pending_index: usize) {
    with_chain_mut(&account_id, chain, |chain| {
        chain.remove_pending(pending_index)
    })
    .unwrap_or_else(revert);
}

#[update(guard = "caller_is_signer")]
pub async fn account_swap_btc_to_ckbtc(
    account_id: AccountId,
    network: BtcNetwork,
    amount: Satoshi,
) -> BtcPending {
    let btc = with_chain(&account_id, &ChainEnum::BTC(network), |chain| chain.btc())
        .unwrap_or_else(revert)
        .unwrap_or_else(revert);

    let result = btc.swap_to_ckbtc(amount).await;

    match result {
        Ok(pending) => {
            with_chain_mut(&account_id, ChainEnum::BTC(network), |chain| {
                chain.add_pending(pending.clone().into())
            })
            .unwrap_or_else(revert);

            pending
        }
        Err(err) => revert(err),
    }
}

#[update(guard = "caller_is_signer")]
pub async fn account_swap_ckbtc_to_btc(
    account_id: AccountId,
    network: BtcNetwork,
    retrieve_address: String,
    amount: Satoshi,
) -> TransferBlockIndex {
    let ckbtc = with_chain(&account_id, &ChainEnum::CKBTC(network), |chain| {
        chain.ckbtc()
    })
    .unwrap_or_else(revert)
    .unwrap_or_else(revert);

    let result = ckbtc.swap_to_btc(retrieve_address, amount).await;

    match result {
        Ok(result) => {
            let block_index = result.block_index;

            with_chain_mut(&account_id, ChainEnum::CKBTC(network), |chain| {
                let pending = PendingEnum::new_ckbtc(block_index, None);

                chain.add_pending(pending)
            })
            .unwrap_or_else(revert);

            block_index
        }
        Err(err) => revert(err),
    }
}

#[update(guard = "caller_is_signer")]
pub async fn account_top_up_and_notify(
    account_id: AccountId,
    amount: ICPToken,
    canister_id: Option<CanisterId>,
) -> Result<Cycles, String> {
    let icp = with_chain(&account_id, &ChainEnum::ICP, |chain| chain.icp())
        .unwrap_or_else(revert)
        .unwrap_or_else(revert);

    let canister_id = canister_id.unwrap_or(ic_cdk::id());

    let block_index = icp.top_up(canister_id, amount).await.unwrap_or_else(revert);

    let notify_result = icp.notify_top_up(canister_id, block_index).await.unwrap();

    match notify_result {
        NotifyTopUpResult::Ok(cycles) => Ok(cycles),
        NotifyTopUpResult::Err(err) => {
            with_chain_mut(&account_id, ChainEnum::ICP, |chain| {
                let pending = PendingEnum::new_icp(block_index, canister_id.to_string());

                chain.add_pending(pending)
            })
            .unwrap();

            Err(err.to_string())
        }
    }
}

#[update(guard = "caller_is_signer")]
pub async fn account_create_address(account_id: AccountId, chain_enum: ChainEnum) {
    let mut ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let ecdsa = match chain_enum {
        ChainEnum::BTC(_) | ChainEnum::EVM(_) => {
            if !ledger.is_public_key_set() {
                let ecdsa = ledger
                    .subaccount
                    .ecdsa_public_key()
                    .await
                    .unwrap_or_else(revert);

                ledger
                    .set_ecdsa_public_key(ecdsa.clone())
                    .unwrap_or_else(revert);

                Some(ecdsa)
            } else {
                None
            }
        }
        _ => None,
    };

    let chain = ledger
        .new_chain(chain_enum.clone())
        .await
        .unwrap_or_else(revert);

    // Check if the chain is ckbtc and update balance for any pending balance
    if chain_enum.is_ckbtc() {
        // We don't care about the result. If it fails, it will be retried later
        let _ = chain.ckbtc().unwrap().update_balance();
    }

    with_ledger_mut(&account_id, |ledger| {
        if let Some(ecdsa) = ecdsa {
            ledger.set_ecdsa_public_key(ecdsa).unwrap_or_else(revert);
        }

        ledger.insert_chain(chain_enum, chain)
    })
    .unwrap_or_else(revert);
}

#[update(guard = "caller_is_signer")]
pub async fn account_btc_fees(network: BtcNetwork, num_blocks: u8) -> u64 {
    let rate = network.fee_rate(num_blocks).await;

    match rate {
        Ok(rate) => rate,
        Err(err) => revert(err),
    }
}

// QUERY

#[query(guard = "caller_is_signer")]
pub fn get_processed_list() -> ProcessedOperations {
    with_processed_operation(|s| s.processed_list())
}

// UPDATE

#[update(guard = "caller_is_signer")]
pub async fn response(
    request_id: OperationId,
    answer: Response,
) -> Result<ProcessedOperation, String> {
    let caller = ic_cdk::caller();

    let request = with_pending_operation_mut(&request_id, |request| {
        if request.is_expired() {
            return request.clone();
        }

        request.response(caller, answer).unwrap_or_else(revert);

        request.clone()
    })
    .unwrap_or_else(revert);

    if request.is_failed() {
        let processed = ProcessedOperation::from(request);

        if let Err(err) = with_processed_operation_mut(|s| s.add(request_id, processed.clone())) {
            return Err(err.to_string());
        }

        return Ok(processed);
    }

    if request.is_confirmed() {
        let processed = request.execute().await;

        if let Err(err) = with_processed_operation_mut(|s| s.add(request_id, processed.clone())) {
            return Err(err.to_string());
        }

        return Ok(processed);
    }

    Ok(request.into())
}

#[update(guard = "caller_is_signer")]
pub fn reset_accounts() {
    with_wallet_mut(|s| s.reset_accounts());
}

#[query(guard = "caller_is_signer")]
fn setting_and_signer() -> WalletSettingsAndSigners {
    let settings = with_setting(|s| s.clone());
    let signers = with_users(|s| s.clone());

    WalletSettingsAndSigners { settings, signers }
}

#[update(guard = "caller_is_admin")]
async fn add_controller_and_update(
    controller_id: ControllerId,
    name: String,
    metadata: Option<Metadata>,
) {
    let controller = WalletController::new(name, metadata);

    let mut settings = with_setting(|s| s.clone());

    settings
        .add_controller_and_update(controller_id, controller)
        .await
        .unwrap_or_else(revert);

    with_wallet_mut(|w| w.set_setting(settings));
}

#[update(guard = "caller_is_admin")]
async fn update_controller(controller_map: WalletControllerMap) -> WalletControllerMap {
    let mut settings = with_setting(|s| s.clone());

    settings
        .update_controller_and_update(controller_map)
        .await
        .unwrap_or_else(revert);

    with_wallet_mut(|w| w.set_setting(settings));

    with_setting(|s| s.controllers().clone())
}

#[update(guard = "caller_is_admin")]
async fn update_settings() {
    let mut settings = with_setting(|s| s.clone());

    settings.update_settings().await.unwrap_or_else(revert);

    with_wallet_mut(|w| w.set_setting(settings));
}

#[update(guard = "caller_is_signer")]
async fn refresh_settings() {
    let mut settings = with_setting(|s| s.clone());

    settings.refresh_settings().await.unwrap_or_else(revert);

    with_wallet_mut(|w| w.set_setting(settings));
}

#[update(guard = "caller_is_signer")]
fn add_setting_metadata(key: String, value: String) {
    with_setting_mut(|s| s.add_metadata(key, value));
}

#[update(guard = "caller_is_signer")]
fn remove_setting_metadata(key: String) {
    with_setting_mut(|s| s.remove_metadata(&key));
}

// QUERY ---------------------------------------------------------------------
#[query(guard = "caller_is_signer")]
pub fn get_pending_list() -> PendingOperations {
    with_operation(|s| s.pending_list())
}

#[query(guard = "caller_is_signer")]
pub fn is_connected() -> bool {
    let caller = ic_cdk::caller();

    with_verified_user(caller, |signer| signer.is_canister()).is_ok()
}

// UPDATE ---------------------------------------------------------------------
#[update(guard = "caller_is_signer")]
pub fn request_maker(
    request: Operation,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    let caller = ic_cdk::caller();

    let allowed_signers = with_users_who_can_operate(&request, |signer_ids| {
        if !signer_ids.contains(&caller) {
            return revert(OperationError::AccessDenied);
        }

        signer_ids.clone()
    });

    let request_args = RequestArgs {
        allowed_signers,
        request: request.into(),
        version: version(),
        reason,
        deadline,
    };

    with_operation_mut(|s| {
        let new_request = s.new_request(caller, request_args);
        s.add(new_request)
    })
}

#[update(guard = "caller_is_admin")]
pub fn request_add_signer(
    request: AddUser,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    request_maker(request.into(), reason, deadline)
}

#[update]
pub fn request_connect(name: String) -> OperationId {
    let signer_id = ic_cdk::caller();

    let request = AddUser {
        role: Role::new(name.clone(), AccessLevel::Canister),
        expires_at: None,
        threshold: None,
        signer_id,
        name,
    };

    with_operation(|s| {
        // check if the request is already in the pending list
        let pending_list = s.pending_list();

        for pending_request in pending_list.iter() {
            if pending_request.request == Operation::AddUser(request.clone()) {
                return revert("Already Pending!");
            }
        }
    });

    if with_verified_user(signer_id, |signer| signer.is_canister()).is_ok() {
        return revert("Already connected!");
    }

    request_maker(
        request.into(),
        format!("Connecting to B3Payment for making payment!"),
        None,
    )
}

#[update(guard = "caller_is_admin")]
pub fn request_update_settings(
    request: UpdateCanisterSettings,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    request_maker(request.into(), reason, deadline)
}

#[update(guard = "caller_is_admin")]
pub fn request_account_rename(
    request: RenameAccount,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    request_maker(request.into(), reason, deadline)
}

#[update(guard = "caller_is_admin")]
pub fn request_create_account(
    request: CreateAccount,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    request_maker(request.into(), reason, deadline)
}

#[update(guard = "caller_is_admin")]
pub fn request_delete_account(
    request: RemoveAccount,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    request_maker(request.into(), reason, deadline)
}

#[update(guard = "caller_is_admin")]
pub fn request_transfer_icp(
    request: IcpTransfer,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    request_maker(request.into(), reason, deadline)
}

#[update(guard = "caller_is_admin")]
pub fn request_transfer_btc(
    request: BtcTransfer,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    request_maker(request.into(), reason, deadline)
}

#[update(guard = "caller_is_signer")]
pub fn request_send(
    request: SendToken,
    reason: String,
    deadline: Option<NanoTimeStamp>,
) -> OperationId {
    request_maker(request.into(), reason, deadline)
}

#[update(guard = "caller_is_signer")]
pub async fn request_upgrade_canister(wasm_version: String) -> OperationId {
    let upgrade_request = with_wasm(|w| UpgradeCanister {
        wasm_hash_string: w.generate_hash_string(),
        wasm_version,
    });

    upgrade_request.validate_request().unwrap_or_else(revert);

    request_maker(upgrade_request.into(), "Upgrade canister".to_string(), None)
}

#[query]
pub fn validate_signer(signer_id: UserId) -> bool {
    with_user(&signer_id, |_| true).is_ok()
}

#[query(guard = "caller_is_admin")]
pub fn get_signers() -> UserMap {
    with_users(|u| u.users().clone())
}

#[update(guard = "caller_is_admin")]
pub fn signer_add(signer_id: UserId, role: Role) -> UserMap {
    let signer = User::from(role);

    with_users_mut(|users| {
        users.add(signer_id.clone(), signer);

        users.get_users()
    })
}

#[update(guard = "caller_is_admin")]
pub fn signer_remove(signer_id: UserId) -> UserMap {
    with_users_mut(|users| {
        users.remove(&signer_id);

        users.get_users()
    })
}

#[update(guard = "caller_is_admin")]
pub async fn init_wallet(args: WalletInititializeArgs) {
    if with_wallet(|w| w.is_initialised()) {
        return revert(WalletError::WalletAlreadyInitialized);
    }

    let mut setting = WalletSettings::new(args.controllers, args.metadata);

    setting.update_settings().await.unwrap_or_else(revert);

    with_wallet_mut(|w| w.init_wallet(setting));
}

#[update(guard = "caller_is_admin")]
async fn upgrage_wallet() {
    let canister_id = ic_cdk::id();
    let wasm_module = with_wasm(|w| {
        if w.is_empty() {
            return revert(WalletError::WasmNotLoaded);
        }
        w.get()
    });

    let args = InstallCodeArgument {
        canister_id,
        wasm_module,
        arg: Vec::new(),
        mode: CanisterInstallMode::Upgrade,
    };

    install_code(args).await.unwrap();
}

#[update(guard = "caller_is_admin")]
pub async fn uninstall_wallet() {
    let canister_id = ic_cdk::id();

    let args = CanisterIdRecord { canister_id };

    uninstall_code(args).await.unwrap();
}

#[update(guard = "caller_is_signer")]
pub async fn status() -> WalletCanisterStatus {
    let canister_id = ic_cdk::api::id();

    let version = version();
    let name = name();

    let canister_status = ic_canister_status(canister_id).await.unwrap_or_else(revert);

    let account_status = with_wallet(|s| s.account_status());
    let status_at = NanoTimeStamp::now();

    WalletCanisterStatus {
        canister_id,
        name,
        version,
        status_at,
        canister_status,
        account_status,
    }
}

#[query]
pub fn canister_cycle_balance() -> u128 {
    ic_cdk::api::canister_balance128()
}

#[query]
pub fn canister_version() -> u64 {
    ic_cdk::api::canister_version()
}

#[query]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[query]
pub fn name() -> String {
    env!("CARGO_PKG_NAME").to_string()
}

#[query(guard = "caller_is_canister_or_admin")]
fn wasm_details() -> WasmDetails {
    with_wasm(|w| {
        let hash = w.generate_hash();
        let size = w.len();

        WasmDetails { hash, size }
    })
}

#[query(guard = "caller_is_signer")]
fn wasm_hash_string() -> String {
    with_wasm(|w| w.generate_hash_string())
}

#[query(guard = "caller_is_signer")]
fn wasm_hash() -> WasmHash {
    with_wasm(|w| w.generate_hash())
}

#[update(guard = "caller_is_canister_or_admin")]
fn load_wasm(blob: Vec<u8>) -> WasmSize {
    with_wasm_mut(|w| w.load(&blob))
}

#[update(guard = "caller_is_admin")]
fn unload_wasm() -> WasmSize {
    with_wasm_mut(|w| w.unload())
}

ic_cdk::export_candid!();
