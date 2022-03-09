use ic_btc_library::{btc_address_str, get_utxos, balance, send};
use ic_cdk_macros::update;
use ic_btc_types::Utxo;

const MIN_CONFIRMATIONS: Option<u32> = Some(0);

#[update]
pub async fn test() -> (String, Vec<Utxo>, u64, Vec<Utxo>, u64) {
    // Returns the UTXOs of the canister's BTC address.
    let before_send_utxos = get_utxos(MIN_CONFIRMATIONS).await;

    // Returns the canister's balance.
    let before_send_balance = balance(MIN_CONFIRMATIONS).await;

    // Send the `amount` of satoshis provided to the `destination` address.
    //
    // Notes:
    //  * Input UTXOs are not being selected in any smart way.
    //  * A dust threshold of 10k satoshis is used.
    send(1_0000_0000, 1_0000, "bcrt1qyepqdteh7w9fhtsrylzghduhj4th5260ae3ywf".to_string()).await;

    // Returns the regtest P2PKH address derived from the private key as a string.
    // P2PKH was chosen for demonstrational purposes. Other address types can also be used.
    (btc_address_str(),

     before_send_utxos, before_send_balance,
     get_utxos(MIN_CONFIRMATIONS).await, balance(MIN_CONFIRMATIONS).await)
}

fn main() {}
