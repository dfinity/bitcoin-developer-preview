import Array "mo:base/Array";
import Debug "mo:base/Debug";
import Error "mo:base/Error";
import Principal "mo:base/Principal";
import Nat64 "mo:base/Nat64";
import Result "mo:base/Result";
import TrieSet "mo:base/TrieSet";

import Common "canister:btc-example-common";
import Types "Types";
import Utils "Utils";


actor class Self(payload : Types.InitPayload) {

    // Actor definition to handle interactions with the BTC canister.
    type BTC = actor {
        // Gets the balance from the BTC canister.
        get_balance : Types.GetBalanceRequest -> async Types.GetBalanceResponse;
        // Retrieves the UTXOs from the BTC canister.
        get_utxos : Types.GetUtxosRequest -> async Types.GetUtxosResponse;
        // Sends a transaction to the BTC canister.
        send_transaction : (Types.SendTransactionRequest) -> async Types.SendTransactionResponse;
    };

    // The canister's private key in "Wallet Import Format".  
    let PRIVATE_KEY_WIF : Text = "L2C1QgyKqNgfV7BpEPAm6PVn2xW8zpXq6MojSbWdH18nGQF2wGsT";
    // Used to interact with the BTC canister.
    let btc : BTC = actor(Principal.toText(payload.bitcoin_canister_id));

    let used_outpoints : Utils.OutPointSet = Utils.OutPointSet();

    // Retrieves the BTC address using the common canister.
    public func btc_address() : async Text {
        await Common.get_p2pkh_address(PRIVATE_KEY_WIF, #Regtest)
    };

    // Retrieves the canister's balance from the BTC canister.
    public func balance() : async Result.Result<Types.Satoshi, ?Types.GetBalanceError> {
        let address : Text = await btc_address();
        switch (await btc.get_balance({ address=address; min_confirmations=?0 })) {
            case (#Ok(satoshi)) {
                #ok(satoshi)
            };
            case (#Err(err)) {
                #err(err)
            };
        }
    };

    // Used to retrieve the UTXOs and process the response.
    func get_utxos_internal(address : Text) : async Result.Result<Types.GetUtxosData, ?Types.GetUtxosError> {
        let result = await btc.get_utxos({
            address=address;
            min_confirmations=?0
        });
        switch (result) {
            case (#Ok(response)) {
                #ok(response)
            };
            case (#Err(err)) {
                #err(err)
            };
        }
    };

    // Exposes the `get_utxos_internal` as and endpoint.
    public func get_utxos() : async Result.Result<Types.GetUtxosData, ?Types.GetUtxosError> {
        let address : Text = await btc_address();
        await get_utxos_internal(address)
    };

    func is_used_outpoint(utxo : Types.Utxo) : Bool {
        not used_outpoints.contains(utxo.outpoint) 
    };

    // Allows Bitcoin to be sent from the canister to a BTC address.
    public func send(amount: Types.Satoshi, destination: Text) : async Result.Result<(), Types.SendError> {
        // Assuming a fixed fee of 10k satoshis.
        let fees : Nat64 = 10_000;
        let source : Text = await btc_address();
        let utxos_response = await get_utxos_internal(source);
        let utxos_data = switch (utxos_response) {
            case (#ok(data)) {
                data 
            };
            case (#err(?error)) {
                return #err(error);
            };
            case (#err(null)) {
                return #err(#Unknown);
            };
        };

        Debug.print("UTXOs");
        Debug.print(debug_show(utxos_data.utxos));

        let filtered_utxos = Array.filter(utxos_data.utxos, is_used_outpoint);
        Debug.print("Filtered UTXOs");
        Debug.print(debug_show(filtered_utxos));

        let (built_transaction, used_utxo_indices) = await Common.build_transaction(filtered_utxos, source, destination, amount, fees);

        Debug.print(debug_show(used_utxo_indices));
        for (index in used_utxo_indices.vals()) {
            let i : Nat = Nat64.toNat(index);
            Debug.print(debug_show(utxos_data.utxos[i]));
            let t = Utils.hashOutPoint(utxos_data.utxos[i].outpoint);
            Debug.print(debug_show(t));
            used_outpoints.add(utxos_data.utxos[i].outpoint);
        };

        let signed_transaction = await Common.build_and_sign_transaction(PRIVATE_KEY_WIF, utxos_data.utxos, source, destination, amount, fees);
        let send_transaction_response = await btc.send_transaction({ transaction=signed_transaction });
        switch (send_transaction_response) {
            case (#Ok) {
                #ok(())
            };
            case (#Err(?error)) {
                #err(error);
            };
            case (#Err(null)) {
                #err(#Unknown);
            };
        }
    };
};
