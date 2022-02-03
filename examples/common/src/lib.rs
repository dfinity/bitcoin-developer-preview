pub mod types;

use bitcoin::{
    blockdata::script::Builder,
    hashes::Hash,
    secp256k1::{Message, Secp256k1},
    Address, AddressType, Network, OutPoint, PrivateKey, Script, SigHashType, Transaction, TxIn,
    TxOut, Txid,
};
use ic_btc_types::Utxo;
use ic_cdk::print;

// The signature hash type that is always used.
const SIG_HASH_TYPE: SigHashType = SigHashType::All;

pub fn get_p2pkh_address(private_key: &PrivateKey, network: Network) -> Address {
    let public_key = private_key.public_key(&Secp256k1::new());
    Address::p2pkh(&public_key, network)
}

// Builds a transaction that sends the given `amount` of satoshis to the `destination` address.
pub fn build_transaction(
    utxos: Vec<Utxo>,
    source: Address,
    destination: Address,
    amount: u64,
    fees: u64,
) -> Result<Transaction, String> {
    // Assume that any amount below this threshold is dust.
    const DUST_THRESHOLD: u64 = 10_000;

    // Select which UTXOs to spend. For now, we naively spend the first available UTXOs,
    // even if they were previously spent in a transaction.
    let mut utxos_to_spend = vec![];
    let mut total_spent = 0;
    for utxo in utxos.into_iter() {
        total_spent += utxo.value;
        utxos_to_spend.push(utxo);
        if total_spent >= amount + fees {
            // We have enough inputs to cover the amount we want to spend.
            break;
        }
    }

    print(&format!("UTXOs to spend: {:?}", utxos_to_spend));

    if total_spent < amount {
        return Err("Insufficient balance".to_string());
    }

    let inputs: Vec<TxIn> = utxos_to_spend
        .into_iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: Txid::from_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: 0xffffffff,
            witness: Vec::new(),
            script_sig: Script::new(),
        })
        .collect();

    let mut outputs = vec![TxOut {
        script_pubkey: destination.script_pubkey(),
        value: amount,
    }];

    let remaining_amount = total_spent - amount - fees;

    if remaining_amount >= DUST_THRESHOLD {
        outputs.push(TxOut {
            script_pubkey: source.script_pubkey(),
            value: remaining_amount,
        });
    }

    Ok(Transaction {
        input: inputs,
        output: outputs,
        lock_time: 0,
        version: 2,
    })
}

/// Sign a bitcoin transaction given the private key and the source address of the funds.
///
/// Constraints:
/// * All the inputs are referencing outpoints that are owned by `src_address`.
/// * `src_address` is a P2PKH address.
pub fn sign_transaction(
    mut transaction: Transaction,
    private_key: PrivateKey,
    src_address: Address,
) -> Transaction {
    // Verify that the address is P2PKH. The signature algorithm below is specific to P2PKH.
    match src_address.address_type() {
        Some(AddressType::P2pkh) => {}
        _ => panic!("This demo supports signing p2pkh addresses only."),
    };

    let secp = Secp256k1::new();
    let txclone = transaction.clone();
    let public_key = private_key.public_key(&Secp256k1::new());

    for (index, input) in transaction.input.iter_mut().enumerate() {
        let sighash =
            txclone.signature_hash(index, &src_address.script_pubkey(), SIG_HASH_TYPE.as_u32());

        let signature = secp
            .sign(
                &Message::from_slice(&sighash[..]).unwrap(),
                &private_key.key,
            )
            .serialize_der();

        let mut sig_with_hashtype = signature.to_vec();
        sig_with_hashtype.push(SIG_HASH_TYPE.as_u32() as u8);
        input.script_sig = Builder::new()
            .push_slice(sig_with_hashtype.as_slice())
            .push_slice(public_key.to_bytes().as_slice())
            .into_script();
        input.witness.clear();
    }

    transaction
}
