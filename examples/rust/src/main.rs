use bitcoin::{util::psbt::serialize::Serialize, Address, Network, PrivateKey};
use example_common::{build_transaction, get_p2pkh_address, sign_transaction};
use ic_btc_types::{
    GetBalanceError, GetBalanceRequest, GetUtxosError, GetUtxosRequest, GetUtxosResponse, OutPoint,
    SendTransactionRequest, Utxo,
};
use ic_cdk::{
    api::call::RejectionCode,
    call,
    export::{
        candid::{CandidType, Deserialize},
        Principal,
    },
    print, trap,
};
use ic_cdk_macros::{init, query, update};
use std::{cell::RefCell, collections::HashSet, str::FromStr};

// A private key in WIF (wallet import format). This is only for demonstrational purposes.
// When the Bitcoin integration is released on mainnet, canisters will have the ability
// to securely generate ECDSA keys.
const BTC_PRIVATE_KEY_WIF: &str = "L2C1QgyKqNgfV7BpEPAm6PVn2xW8zpXq6MojSbWdH18nGQF2wGsT";

thread_local! {
    static BTC_PRIVATE_KEY: RefCell<PrivateKey> =
        RefCell::new(PrivateKey::from_wif(BTC_PRIVATE_KEY_WIF).unwrap());

    // The ID of the bitcoin canister that is installed locally.
    // The value here is initialized with a dummy value, which will be overwritten in `init`.
    static BTC_CANISTER_ID: RefCell<Principal> = RefCell::new(Principal::management_canister());

    // A cache of spent outpoints. Needed to avoid double spending.
    static SPENT_TXOS: RefCell<HashSet<OutPoint>> = RefCell::new(HashSet::new());
}

#[derive(CandidType, Deserialize)]
struct InitPayload {
    bitcoin_canister_id: Principal,
}

#[init]
fn init(payload: InitPayload) {
    BTC_CANISTER_ID.with(|id| {
        // Set the ID fo the bitcoin canister.
        id.replace(payload.bitcoin_canister_id);
    })
}

/// Returns the regtest P2PKH address derived from the private key as a string.
/// P2PKH was chosen for demonstrational purposes. Other address types can also be used.
#[query(name = "btc_address")]
pub fn btc_address_str() -> String {
    btc_address().to_string()
}

/// Returns the UTXOs of the canister's BTC address.
#[update]
pub async fn get_utxos() -> Vec<Utxo> {
    let btc_canister_id = BTC_CANISTER_ID.with(|id| *id.borrow());
    #[allow(clippy::type_complexity)]
    let res: Result<
        (Result<GetUtxosResponse, Option<GetUtxosError>>,),
        (RejectionCode, String),
    > = call(
        btc_canister_id,
        "get_utxos",
        (GetUtxosRequest {
            address: btc_address_str(),
            min_confirmations: Some(0),
        },),
    )
    .await;

    match res {
        // Return the UTXOs to the caller.
        Ok((Ok(data),)) => data.utxos,

        // The call to `get_utxos` returned an error.
        Ok((Err(err),)) => trap(&format!("Received error from Bitcoin canister: {:?}", err)),

        // The call to `get_utxos` was rejected.
        // This is only likely to happen if there's a bug in the bitcoin canister.
        Err((rejection_code, message)) => trap(&format!(
            "Received a reject from Bitcoin canister.\nRejection Code: {:?}\nMessage: '{}'",
            rejection_code, message
        )),
    }
}

/// Returns the canister's balance.
#[update]
pub async fn balance() -> u64 {
    let btc_canister_id = BTC_CANISTER_ID.with(|id| *id.borrow());
    let res: Result<(Result<u64, GetBalanceError>,), (RejectionCode, String)> = call(
        btc_canister_id,
        "get_balance",
        (GetBalanceRequest {
            address: btc_address_str(),
            min_confirmations: Some(0),
        },),
    )
    .await;

    match res {
        // Return the balance to the caller.
        Ok((Ok(balance),)) => balance,

        // The call to `get_balance` returned an error.
        Ok((Err(err),)) => trap(&format!("Received error from Bitcoin canister: {:?}", err)),

        // The call to `get_balance` was rejected.
        // This is only likely to happen if there's a bug in the bitcoin canister.
        Err((rejection_code, message)) => trap(&format!(
            "Received a reject from Bitcoin canister.\nRejection Code: {:?}\nMessage: '{}'",
            rejection_code, message
        )),
    }
}

/// Send the `amount` of satoshis provided to the `destination` address.
///
/// Notes:
///  * Fees are hard-coded to 10k satoshis.
///  * Input UTXOs are not being selected in any smart way.
///  * A dust threshold of 10k satoshis is used.
#[update]
pub async fn send(amount: u64, destination: String) {
    let fees: u64 = 10_000;

    if amount <= fees {
        trap("Amount must be higher than the fee of 10,000 satoshis")
    }

    let destination = match Address::from_str(&destination) {
        Ok(destination) => destination,
        Err(_) => trap("Invalid destination address"),
    };

    // Fetch our UTXOs.
    let utxos = get_utxos().await;

    // Remove any spent UTXOs that were already used for past transactions.
    let utxos = utxos
        .into_iter()
        .filter(|utxo| SPENT_TXOS.with(|spent_txos| !spent_txos.borrow().contains(&utxo.outpoint)))
        .collect();

    let spending_transaction = build_transaction(utxos, btc_address(), destination, amount, fees)
        .unwrap_or_else(|err| {
            trap(&format!("Error building transaction: {}", err));
        });

    // Cache the spent outputs to not use them for future transactions.
    for tx_in in spending_transaction.input.iter() {
        SPENT_TXOS.with(|spent_txos| {
            print(&format!("Caching {:?}", tx_in.previous_output.txid.to_vec()));
            spent_txos.borrow_mut().insert(OutPoint {
                txid: tx_in.previous_output.txid.to_vec(),
                vout: tx_in.previous_output.vout,
            })
        });
    }

    print(&format!(
        "Transaction to sign: {}",
        hex::encode(spending_transaction.serialize())
    ));

    // Sign transaction
    let private_key = BTC_PRIVATE_KEY.with(|private_key| *private_key.borrow());
    let signed_transaction = sign_transaction(spending_transaction, private_key, btc_address());

    let signed_transaction_bytes = signed_transaction.serialize();
    print(&format!(
        "Signed transaction: {}",
        hex::encode(signed_transaction_bytes.clone())
    ));

    let btc_canister_id = BTC_CANISTER_ID.with(|id| *id.borrow());

    print("Sending transaction");

    let _: Result<(), (RejectionCode, String)> = call(
        btc_canister_id,
        "send_transaction",
        (SendTransactionRequest {
            transaction: signed_transaction_bytes,
        },),
    )
    .await;
}

// Returns the regtest P2PKH address derived from the private key.
fn btc_address() -> Address {
    BTC_PRIVATE_KEY.with(|private_key| get_p2pkh_address(&private_key.borrow(), Network::Regtest))
}

fn main() {}
