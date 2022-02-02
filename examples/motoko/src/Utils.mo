import Array "mo:base/Array";
import Blob "mo:base/Blob";
import Hash "mo:base/Hash";
import Nat "mo:base/Nat";
import Nat8 "mo:base/Nat8";
import Nat32 "mo:base/Nat32";
import TrieSet "mo:base/TrieSet";

import Types "Types";

module {

    func nat8ToNat32 (n: Nat8) : Nat32 {
        Nat32.fromNat(Nat8.toNat(n))
    };

    /// Returns a hash obtained by using the `djb2` algorithm from http://www.cse.yorku.ca/~oz/hash.html
    ///
    /// This function is _good enough_ for use in a hash-table but it's not a cryptographic hash function!
    public func hashOutPoint(outpoint : Types.OutPoint) : Hash.Hash {
        let outpoint_data : [Nat32] = Array.append(Array.map(Blob.toArray(outpoint.txid), nat8ToNat32), [outpoint.vout]);
        var x : Nat32 = 5381;
        for (c in outpoint_data.vals()) {
            x := ((x << 5) +% x) +% c;
        };
        x
    };

    public func areOutPointsEqual(o1 : Types.OutPoint, o2 : Types.OutPoint) : Bool {
        if (o1.vout != o2.vout) {
            return false;
        };

        Blob.equal(o1.txid, o2.txid)
    };

    public class OutPointSet () {

        var trie_set : TrieSet.Set<Types.OutPoint> = TrieSet.empty();

        public func add(outpoint : Types.OutPoint) {
            let s2 = TrieSet.put(trie_set, outpoint, hashOutPoint(outpoint), areOutPointsEqual);
            trie_set := s2;
        };

        public func contains(outpoint : Types.OutPoint) : Bool {
            TrieSet.mem(trie_set, outpoint, hashOutPoint(outpoint), areOutPointsEqual)
        }

    };

}
