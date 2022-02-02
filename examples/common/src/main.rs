//! A simple candid API for Motoko canisters
//!
//! Some fundamental crypto primitives are still missing in Motoko, and that
//! makes some fundamental operations, like computing a Bitcoin address and
//! signing a transaction impossible in Motoko.
//!
//! The following is a work-around until the crypto primitives become available
//! to canisters. We wrap all the functionality requiring cryptography into a
//! canister, and Motoko developers can deploy this canister and interact with it
//! for address computation and transaction signing.
use bitcoin::{util::psbt::serialize::Serialize, Address, Network, PrivateKey};
use ic_btc_types::Utxo;
use ic_cdk::export::candid::{candid_method, CandidType, Deserialize};
use ic_cdk_macros::query;
use std::str::FromStr;

#[derive(CandidType, Deserialize, Copy, Clone)]
pub enum NetworkCandid {
    Bitcoin,
    Regtest,
    Testnet,
    Signet,
}

// Returns the P2PKH address of the given private key.
#[query]
#[candid_method(query)]
fn get_p2pkh_address(private_key_wif: String, network: NetworkCandid) -> String {
    let private_key = PrivateKey::from_wif(&private_key_wif).expect("Invalid private key WIF");
    let network = match network {
        NetworkCandid::Bitcoin => Network::Bitcoin,
        NetworkCandid::Testnet => Network::Testnet,
        NetworkCandid::Regtest => Network::Regtest,
        NetworkCandid::Signet => Network::Signet,
    };
    example_common::get_p2pkh_address(&private_key, network).to_string()
}

// Returns the serialized bytes of the signed transaction.
#[query]
#[candid_method(query)]
fn build_and_sign_transaction(
    private_key_wif: String,
    utxos: Vec<Utxo>,
    source_address: String,
    destination_address: String,
    amount: u64,
    fees: u64,
) -> Vec<u8> {
    let private_key = PrivateKey::from_wif(&private_key_wif).expect("Invalid private key WIF");
    let source_address = Address::from_str(&source_address).expect("Invalid source address");
    let destination_address =
        Address::from_str(&destination_address).expect("Invalid destination address");
    let tx = example_common::build_transaction(
        utxos,
        source_address.clone(),
        destination_address,
        amount,
        fees,
    )
    .expect("Building transaction failed");
    example_common::sign_transaction(tx, private_key, source_address).serialize()
}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_candid_interface_compatibility() {
        use candid::types::subtype::{subtype, Gamma};
        use candid::types::Type;
        use ic_cdk::export::candid::{self};
        use std::io::Write;
        use std::path::PathBuf;

        candid::export_service!();

        let actual_interface = __export_service();
        println!("Generated DID:\n {}", actual_interface);
        let mut tmp = tempfile::NamedTempFile::new().expect("failed to create a temporary file");
        write!(tmp, "{}", actual_interface).expect("failed to write interface to a temporary file");
        let (mut env1, t1) =
            candid::pretty_check_file(tmp.path()).expect("failed to check generated candid file");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("candid.did");
        let (env2, t2) =
            candid::pretty_check_file(path.as_path()).expect("failed to open candid.did file");

        let (t1_ref, t2) = match (t1.as_ref().unwrap(), t2.unwrap()) {
            (Type::Class(_, s1), Type::Class(_, s2)) => (s1.as_ref(), *s2),
            (Type::Class(_, s1), s2 @ Type::Service(_)) => (s1.as_ref(), s2),
            (s1 @ Type::Service(_), Type::Class(_, s2)) => (s1, *s2),
            (t1, t2) => (t1, t2),
        };

        let mut gamma = Gamma::new();
        let t2 = env1.merge_type(env2, t2);
        subtype(&mut gamma, &env1, t1_ref, &t2)
            .expect("canister interface is not compatible with the candid.did file");
    }
}
