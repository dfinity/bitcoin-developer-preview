// This module contains types needed to interact with the BTC and Common canisters
// along with the typing needed for this canister.
module Types {

    // Used to initialize the example dapp.
    public type InitPayload = {
        bitcoin_canister_id : Principal;
    };

    public type SendError = {
        #MalformedSourceAddress;
        #MalformedDestinationAddress;
        #MalformedTransaction;
        #InsufficientBalance;
        #InvalidPrivateKeyWif;
        #Unknown;
    };

    // Types to interact with the Bitcoin and Common canisters.

    // A single unit of Bitcoin
    public type Satoshi = Nat64;

    // The type of Bitcoin network the dapp will be interacting with.
    public type Network = {
        #Bitcoin;
        #Regtest;
        #Testnet;
        #Signet;
    };

    // A reference to a transaction output.
    public type OutPoint = {
        txid : Blob;
        vout : Nat32;
    };

    // An unspent transaction output.
    public type Utxo = {
        outpoint : OutPoint;
        value : Satoshi;
        height : Nat32;
        confirmations : Nat32;
    };

    public type GetUtxosRequest = {
        address : Text;
        min_confirmations : ?Nat32;
    };

    public type GetUtxosData = {
        utxos : [Utxo];
        total_count : Nat32;
    };

    public type GetUtxosResponse = {
        #Ok : GetUtxosData;
        #Err : ?GetUtxosError;
    };

    public type GetUtxosError = {
        #MalformedAddress;
    };

    public type GetBalanceRequest = {
        address : Text;
        min_confirmations : ?Nat32;
    };

    public type GetBalanceResponse = {
        #Ok : Nat64;
        #Err : ?GetBalanceError;
    };

    public type GetBalanceError = {
        #MalformedAddress;
    };

    public type SendTransactionResponse = {
        #Ok;
        #Err : ?SendTransactionError;
    };

    public type SendTransactionRequest = {
        transaction : Blob;
    };

    public type SendTransactionError = {
        #MalformedTransaction;
    };

}
