//! # Aqua Guardian
//!
//! This crate is the part of the [`guardian`](../guardian/index.html) responsible for shared data strucures
//!
//! It is part of the [aqua project](https://aqua-protocol.org/).
//!
//!
#[cfg(test)]
use signing::{sign_revision_hash, SimpleSigner};

/// Used for cryptographic Hashes
pub mod crypt {
    pub type Hasher = sha3::Sha3_512;
    pub type Hash = sha3::digest::Output<Hasher>;
    pub use sha3::*;
}

/// Implements the types that are internally used by the guardian.
pub mod custom_types;

/// Accumulates all *important* data types
pub mod prelude {
    pub use super::crypt::{self, digest::Digest};
    pub use super::custom_types::*;
    pub use ethaddr::Address;
}

/// Implements the lookup of Exported aqua chains on the Etherium network
pub mod eth_lookup {
    use std::future::Future;

    #[non_exhaustive]
    pub enum EthChain {
        Main,
        #[deprecated]
        Goerli,
    }

    pub struct WitnessEventInfo {
        pub timestamp: chrono::NaiveDateTime,
        pub data: crate::crypt::Hash,
    }

    pub trait EthLookup {
        type Error: std::error::Error;
        fn lookup(
            chain: EthChain,
            transaction_hash: [u8; 32],
        ) -> impl Future<Output = Result<WitnessEventInfo, Self::Error>> + Send;
    }
}

/// Authorisation module for a secure connection to a ToDo
pub mod auth {
    use std::future::Future;

    /// Represents an authorisation request
    pub trait Auth {
        type AuthReq;
        /// Requests an authorisation, returning the result
        fn auth(auth_req: Self::AuthReq) -> impl Future<Output = String> + Send;
    }
}

/// API for interaction with a persistent storage solution
pub mod storage {
    use std::{fmt::Debug, future::Future};

    use crate::custom_types::*;

    pub trait Storage: Sized {
        type Error: std::error::Error + Debug;
        type Context;

        fn get_context(
            &self,
            hash: Hash,
        ) -> impl Future<Output = Result<Self::Context, Self::Error>> + Send;
        fn store(
            &self,
            // hash: Hash,
            rev: Revision,
            context: Self::Context,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send;
        fn read(
            &self,
            hash: Hash,
        ) -> impl Future<Output = Result<Revision, Self::Error>> + Send + Sync;
        fn get_branch(
            &self,
            hash: Hash,
        ) -> impl Future<Output = Result<Branch<Self::Context>, Self::Error>> + Send;
        fn list(&self) -> impl Future<Output = Result<Vec<Hash>, Self::Error>> + Send;
        fn update_handler<F: Fn(Hash, String) + Send + Sync>(
            &self,
            f: F,
        ) -> impl Future<Output = Result<std::convert::Infallible, Self::Error>> + Send;
    }
}

/// Signature encoding
pub mod signing {
    use super::prelude::*;

    pub fn sign_revision_hash<S: Signer>(s: S, verification_hash: Hash) -> Signature {
        let mut k = crypt::Keccak256::default();
        // 3.b add "\x19Ethereum Signed Message:\n177I sign the following page verification_hash: [0x" to hasher {k}
        k.update(
            "\x19Ethereum Signed Message:\n177I sign the following page verification_hash: [0x",
        );
        // 3.c add rev.metadata.verification_hash to hasher {k}
        k.update(verification_hash.to_stackstr());
        // 3.d add "]" to hasher {k}
        k.update("]");
        // 3.e parse output of hasher {k} as secp256k1 message
        let message = libsecp256k1::Message::parse(&k.finalize().into());

        s.sign(&message)
    }

    pub trait Signer {
        fn sign(&self, msg: &libsecp256k1::Message) -> Signature;
        fn identity(&self) -> PublicKey;
    }
    impl<T: Signer> Signer for &T {
        fn sign(&self, msg: &libsecp256k1::Message) -> Signature {
            T::sign(self, msg)
        }
        fn identity(&self) -> PublicKey {
            T::identity(self)
        }
    }

    pub struct SimpleSigner(pub libsecp256k1::SecretKey);
    impl TryFrom<[u8; 32]> for SimpleSigner {
        type Error = libsecp256k1::Error;
        fn try_from(value: [u8; 32]) -> Result<Self, Self::Error> {
            libsecp256k1::SecretKey::parse(&value).map(SimpleSigner)
        }
    }
    #[derive(thiserror::Error, Debug)]
    pub enum PrivateKeyParse {
        #[error("libsecp256k1: {0}")]
        Security(#[from] libsecp256k1::Error),
        #[error("not 0x prefixed")]
        NoPrefix,
        #[error("not hex")]
        NotHex(#[from] hex::FromHexError),
        #[error("false hex")]
        WrongLength,
    }
    impl std::str::FromStr for SimpleSigner {
        type Err = PrivateKeyParse;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let stripped = s.strip_prefix("0x").ok_or(PrivateKeyParse::NoPrefix)?;

            let dec: [u8; 32] = hex::decode(stripped)?
                .try_into()
                .map_err(|_| PrivateKeyParse::WrongLength)?;
            Ok(dec.try_into()?)
        }
    }
    impl Signer for SimpleSigner {
        fn sign(&self, msg: &libsecp256k1::Message) -> Signature {
            libsecp256k1::sign(msg, &self.0).into()
        }
        fn identity(&self) -> PublicKey {
            libsecp256k1::PublicKey::from_secret_key(&self.0).into()
        }
    }
}

#[test]
fn sign_revisions() {
    let s: SimpleSigner = ("0x28475bdbd0425ce597494513b7c4d579d0b366633afd584050610d64971141a7")
        .parse()
        .expect("failed to parse private key");
    let verification_hash =
        "d9e09f8529fed3b909876f34f21c7148d73de01d82f8aee43c52d9ee2601999ddcbf4593a19baac497d9d83bb98c94c2508b8157efafcd6484cbca7c4953af5f".parse();
    let verification_hash = verification_hash.unwrap();
    sign_revision_hash(s, verification_hash);
}
