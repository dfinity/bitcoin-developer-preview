use ic_btc_library::{btc_address_str, get_balance, get_utxos, send, update_transaction};
use ic_btc_types::Utxo;
use ic_cdk::trap;
use ic_cdk_macros::update;

const MIN_CONFIRMATIONS: Option<u32> = Some(0);

#[update]
pub async fn test() -> (String, Vec<Utxo>, u64, String, String) {
    let recipient = String::from("bcrt1qyepqdteh7w9fhtsrylzghduhj4th5260ae3ywf");

    // Send the `amount` with `fees` of satoshis provided to the `destination` address.
    // If `is_modifiable` is set to true, the transaction is repleacable until it got mined in a block
    let txid = send(1_000_000, 100_000, recipient.clone(), true).await;

    // Update a transaction identified by its `txid` with a `new_amount`, `new_fees` and a `new_destination` addresss
    let result =
        update_transaction(txid.clone(), 10_000_000, 1_000_000, recipient.clone()).await;
    let update_txid;
    match result {
        Ok(txid) => update_txid = txid,
        Err(err) => trap(&format!("Error: Update transaction failed: {:?}", err)),
    }

    (
        // Return the base-58 Bitcoin address of the canister
        btc_address_str(),
        // Both getters below always return the same value until a block, with transaction concerning the canister, is mined
        // Return the UTXOs of the canister's BTC address
        get_utxos(MIN_CONFIRMATIONS).await,
        // Return the canister's balance
        get_balance(MIN_CONFIRMATIONS).await,
        // Return the original and updated transactions ids
        txid,
        update_txid,
    )
}

fn main() {}
