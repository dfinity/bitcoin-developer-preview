use ic_btc_library::{
    btc_address_str as ic_btc_library_btc_address_str, get_balance as ic_btc_library_get_balance,
    get_utxos as ic_btc_library_get_utxos, send_transaction as ic_btc_library_send_transaction,
    update_transaction as ic_btc_library_update_transaction,
};
use ic_btc_types::{UpdateTransactionError, Utxo};
use ic_cdk_macros::{query, update};

/// Return the base-58 Bitcoin address of the canister
#[query]
pub fn btc_address() -> String {
    ic_btc_library_btc_address_str().to_string()
}

/// Both getters below always return the same value until a block, with transactions concerning the canister, is mined
/// Returns the UTXOs of the canister's BTC address.
#[update]
pub async fn get_utxos() -> Vec<Utxo> {
    ic_btc_library_get_utxos(Some(0)).await
}

/// Returns the canister's balance.
#[update]
pub async fn get_balance() -> u64 {
    ic_btc_library_get_balance(Some(0)).await
}

/// Send the `amount` with `fees` of satoshis provided to the `destination` address.
/// If `is_modifiable` is set to true, the transaction is replaceable until it gets mined in a block
#[update]
pub async fn send_transaction(
    amount: u64,
    fees: u64,
    destination: String,
    is_modifiable: bool,
) -> String {
    ic_btc_library_send_transaction(amount, fees, destination, is_modifiable).await
}

/// Update a transaction identified by its `tx_id` with a `new_amount`, `new_fees` and a `new_destination` address
#[update]
pub async fn update_transaction(
    tx_id: String,
    new_amount: u64,
    new_fees: u64,
    new_destination: String,
) -> Result<String, UpdateTransactionError> {
    ic_btc_library_update_transaction(tx_id, new_amount, new_fees, new_destination).await
}

fn main() {}
