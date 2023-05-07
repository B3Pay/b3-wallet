use ic_cdk::{api::call::RejectionCode, export::candid::CandidType};

/// Represents errors that can occur when working with the state.
#[derive(CandidType, Debug, PartialEq)]
pub enum SignerError {
    UnknownError,
    SignError(String),
    LedgerError(String),
    GenerateError(String),
    CanisterError(String),
    PublicKeyError(String),
    CanisterStatusError(String),
    CyclesMintingError(String),
    ManagementCanisterError(String),
    MissingEcdsaPublicKey,
    CallerNotAuthorized,
    CallerIsNotOwner,
    CallerIsNotWalletCanister,
    MaximumAccountsReached,
    MaximumDevelopmentAccountsReached,
    MaximumProductionAccountsReached,
    InvalidEcdsaPublicKey,
    PublicKeyMismatch,
    EnvironmentMismatch,
    AccountNotExists,
    AccountAlreadyExists,
    ChainNotFound,
    ChainAlreadyExists,
    TransactionNotFound,
    AccountLimitReached,
    RequestNotExists,
    PasswordHashError,
    PasswordIsInvalid,
    PasswordNotSet,
    TransactionNotPending,
    TransactionAlreadyRemoved,
    InsufficientBalance,
    InvalidSubaccount,
    InvalidPublicKey,
    InvalidAddress,
    TransactionTypeNotFound,
}

impl From<SignerError> for (RejectionCode, String) {
    fn from(error: SignerError) -> Self {
        match error {
            SignerError::LedgerError(msg) => (
                RejectionCode::CanisterError,
                ["Ledger error ", &msg].concat(),
            ),
            SignerError::GenerateError(msg) => (
                RejectionCode::CanisterError,
                ["Generate error ", &msg].concat(),
            ),
            SignerError::CyclesMintingError(msg) => (
                RejectionCode::CanisterError,
                ["Cycles minting error ", &msg].concat(),
            ),
            SignerError::PublicKeyError(msg) => (
                RejectionCode::CanisterError,
                ["Public key error ", &msg].concat(),
            ),
            SignerError::ManagementCanisterError(msg) => (
                RejectionCode::CanisterError,
                ["Management canister error ", &msg].concat(),
            ),
            SignerError::SignError(msg) => {
                (RejectionCode::CanisterError, ["Sign error ", &msg].concat())
            }
            SignerError::CanisterError(msg) => (
                RejectionCode::CanisterError,
                ["Canister error ", &msg].concat(),
            ),
            SignerError::CanisterStatusError(msg) => (
                RejectionCode::CanisterError,
                ["Canister status error ", &msg].concat(),
            ),
            SignerError::UnknownError => (RejectionCode::Unknown, "Unknown error".to_string()),
            SignerError::MissingEcdsaPublicKey => (
                RejectionCode::CanisterError,
                "Missing public key".to_string(),
            ),
            SignerError::CallerNotAuthorized => (
                RejectionCode::CanisterReject,
                "Caller not authorized to perform this action".to_string(),
            ),
            SignerError::CallerIsNotOwner => (
                RejectionCode::CanisterReject,
                "Caller is not owner".to_string(),
            ),
            SignerError::MaximumAccountsReached => (
                RejectionCode::CanisterError,
                "Maximum accounts reached".to_string(),
            ),
            SignerError::CallerIsNotWalletCanister => (
                RejectionCode::CanisterReject,
                "Caller is not wallet canister".to_string(),
            ),
            SignerError::MaximumDevelopmentAccountsReached => (
                RejectionCode::CanisterError,
                "Maximum development accounts reached".to_string(),
            ),
            SignerError::MaximumProductionAccountsReached => (
                RejectionCode::CanisterError,
                "Maximum production accounts reached".to_string(),
            ),
            SignerError::InvalidEcdsaPublicKey => (
                RejectionCode::CanisterError,
                "Invalid ECDSA public key".to_string(),
            ),
            SignerError::AccountNotExists => (
                RejectionCode::CanisterError,
                "Account not found".to_string(),
            ),
            SignerError::AccountAlreadyExists => (
                RejectionCode::CanisterError,
                "Account already exists".to_string(),
            ),
            SignerError::ChainNotFound => {
                (RejectionCode::CanisterError, "Chain not found".to_string())
            }
            SignerError::ChainAlreadyExists => (
                RejectionCode::CanisterError,
                "Chain already exists".to_string(),
            ),
            SignerError::TransactionNotFound => (
                RejectionCode::CanisterError,
                "Transaction not found".to_string(),
            ),
            SignerError::AccountLimitReached => (
                RejectionCode::CanisterError,
                "Account limit reached".to_string(),
            ),
            SignerError::RequestNotExists => (
                RejectionCode::CanisterError,
                "Setting not found".to_string(),
            ),
            SignerError::PasswordHashError => (
                RejectionCode::CanisterError,
                "Password hash error".to_string(),
            ),
            SignerError::PasswordIsInvalid => (
                RejectionCode::CanisterError,
                "Password is invalid".to_string(),
            ),
            SignerError::PasswordNotSet => {
                (RejectionCode::CanisterError, "Password not set".to_string())
            }
            SignerError::PublicKeyMismatch => (
                RejectionCode::CanisterError,
                "Public key mismatch".to_string(),
            ),
            SignerError::EnvironmentMismatch => (
                RejectionCode::CanisterError,
                "Environment mismatch".to_string(),
            ),
            SignerError::TransactionNotPending => (
                RejectionCode::CanisterError,
                "Transaction not pending".to_string(),
            ),
            SignerError::TransactionAlreadyRemoved => (
                RejectionCode::CanisterError,
                "Transaction already removed".to_string(),
            ),

            SignerError::InvalidSubaccount => (
                RejectionCode::CanisterError,
                "Invalid subaccount".to_string(),
            ),
            SignerError::InsufficientBalance => (
                RejectionCode::CanisterError,
                "Insufficient balance".to_string(),
            ),
            SignerError::InvalidPublicKey => (
                RejectionCode::CanisterError,
                "Invalid public key".to_string(),
            ),
            SignerError::InvalidAddress => {
                (RejectionCode::CanisterError, "Invalid address".to_string())
            }
            SignerError::TransactionTypeNotFound => (
                RejectionCode::CanisterError,
                "Transaction type not found".to_string(),
            ),
        }
    }
}
