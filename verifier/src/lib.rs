//#![deny(warnings)]

//! # Aqua Guardian
//!
//! This crate is the part of the [`guardian`](../guardian/index.html) responsible for shared data strucures
//!
//! It is part of the [aqua project](https://aqua-protocol.org/).
//!

pub mod sa;
pub mod v1_1;
pub mod v1_2;

flagset::flags! {
    pub enum HashChainIntegrity: u8 {
        // this could be caught at parse time
        KeyValueMismatch = 1 << 0,
        /// latest_verification_hash missing
        ChainHeadMissing = 1 << 1,
        /// genesis_hash not in revisions
        GenesisHashMissing = 1 << 2,
        /// no path from latest_verification_hash to genesis_hash
        ChainLinkMissing = 1 << 3,
        /// genesis_hash not root
        ChainGenesisIsLink = 1<< 4,
        /// last chain link lists previous
        ChainRootMissing = 1 << 5,
        /// there are revisions not involved in the chain - they should be removed for inout
        UnusedRevisions = 1 << 6,
        /// a revision's integrity is compromised
        RevisionIntegrityFatal = 1 << 7,
    }

    // pub enum RevisionTreeIntegrity: u8 {
    //     TreeRootIsNotGenesisHash,
    //     ParentNodeMissing,
    //     GenesisNodeListsParent,
    // }

    pub enum RevisionIntegrity: u16 {
        NoPrevRevision = 1 << 0,
        PrevVerificationHashNotMatching = 1 << 1,
        NoFile = 1 << 2,
        FileHashNotMatching = 1 << 3,
        ContentHashNotMatching = 1 << 4,
        MetadataHashNotMatching = 1 << 5,
        VerificationHashNotMatching = 1 << 6,

        /// means that no signature field was present, may be ignored if a signature isn't required
        /// if a following revision's verification_hash doesn't match and this flag is set that may imply that the signature has been removed
        NoSignature =  1 << 7,
        SignatureError = 1 << 8,
        PublicKeyNotMatching = 1 << 9,
        SignatureHashNotMatching = 1 << 10,

        /// means that no witness field was present, may be ignored if a witness isn't required
        /// if a following revision's verification_hash doesn't match and this flag is set that may imply that the witness has been removed
        NoWitness = 1 << 11,
        DuplicateMerkleLeaf = 1 << 12,
        VerificationHashNotInMerkleTree = 1 << 13,
        MerkleTreeIncomplete = 1 << 14,
        WitnessHashNotMatching = 1 << 15,
    }
}
