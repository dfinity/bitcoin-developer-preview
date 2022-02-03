use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize)]
pub enum BuildTransactionError {
    InsufficientBalance,
    MalformedDestinationAddress,
    MalformedSourceAddress,
}

#[derive(CandidType, Deserialize)]
pub enum SignTransactionError {
    InvalidPrivateKeyWif,
    MalformedSourceAddress,
    MalformedTransaction,
}
