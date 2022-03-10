use ic_btc_library::{btc_address_str, get_utxos, get_balance, send};
use ic_cdk_macros::update;
use ic_btc_types::Utxo;

const MIN_CONFIRMATIONS: Option<u32> = Some(0);

#[update]
pub async fn test() -> (String, Vec<Utxo>, u64, Vec<Utxo>, u64) {
    // Returns the UTXOs of the canister's BTC address.
    let before_send_utxos = get_utxos(MIN_CONFIRMATIONS).await;

    // Returns the canister's balance.
    let before_send_balance = get_balance(MIN_CONFIRMATIONS).await;

    // Send the `amount` of satoshis provided to the `destination` address.
    // Additional `fees` are sent, 10k in this example
    send(1_0000_0000, 1_0000, "bcrt1qyepqdteh7w9fhtsrylzghduhj4th5260ae3ywf".to_string()).await;

    // Returns the base-58 Bitcoin address of the canister
    (btc_address_str(),

     before_send_utxos, before_send_balance,
     // UTXOs and balance unchanged after the send call because the transaction hasn't had the time to get in a block
     get_utxos(MIN_CONFIRMATIONS).await, get_balance(MIN_CONFIRMATIONS).await)
}

fn main() {}
