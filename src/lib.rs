#![deny(clippy::undocumented_unsafe_blocks, warnings)]
#![allow(clippy::type_complexity)]

//! # Aqua Guardian
//!
//! This crate provides the executable `guardian` to act as a connector and enforcer for Aqua Containers.
//!
//! It is part of the [aqua project](https://aqua-protocol.org/).
//!
//! The guardian is the control structure for all components.
//! The guardian builds up an internal state with the verifier and contract interpreter to be able to share Aqua-Chains based on permissions identified in Contracts.
//! The logic for keeping an updated state, and being able to share and receive Aqua-Chains between Guardians is the core functionality of the Guardian crate.
//!
//! The components are:
//! - [`common`][`guardian_common`] for shared data structures and traits
//! - [`verifier`], responsible for ensuring the integrity of aqua chains
//! - [`contract-interpreter`][`contract_interpreter`] for detecting instructions for the guardian in aqua chains
//! - [`storage-api`][`pkc_api`] for interating with a PKC storage container
//! - [`ethereum-lookup`][`node-eth-lookup`] for looking up witness hashes and block time
//! - [`guardian-api`][`guardian_api`] for interaction between guardians
//!
pub mod contract_generation;
pub mod certificate_generation;

use contract_interpreter::{Contract, ContractEffect, SequencedContract};
use guardian_common::{prelude::*, storage::Storage};
use parking_lot::RwLock;
use std::{
    fmt::Debug,
    sync::{Arc, Weak},
};
use weak_table::WeakValueHashMap;

type RwWeakMap<K, V> = RwLock<WeakValueHashMap<K, Weak<V>>>;

//todo!
pub struct RwWeaakMap<K: Eq + std::hash::Hash, V>(RwWeakMap<K, V>);
impl<K: Eq + std::hash::Hash, V> std::ops::Deref for RwWeaakMap<K, V> {
    type Target = RwWeakMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[allow(clippy::print_in_format_impl)]
impl<K: Eq + std::hash::Hash + std::fmt::Debug, V> Debug for RwWeaakMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_set();
        eprintln!("Debug read: debug");
        for (key, _) in self.0.read().iter() {
            x.entry(key);
        }
        x.finish()
    }
}
impl<K: Eq + std::hash::Hash, V> Default for RwWeaakMap<K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<K: Eq + std::hash::Hash, V> From<RwWeakMap<K, V>> for RwWeaakMap<K, V> {
    fn from(value: RwWeakMap<K, V>) -> Self {
        Self(value)
    }
}

pub struct ContractInfo {
    pub data: Contract,
    pub seqno: Option<u8>,
    /// serves to keep a [`ContractNode`] alive while the corresponding [`StateNode`] still exists
    pub effective: Option<Arc<ContractNode>>,
}

impl Debug for ContractInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractInfo")
            .field("data", &self.data)
            .field("seqno", &self.seqno)
            .field("effective", &self.effective)
            .finish()
    }
}

impl contract_interpreter::ContractInfo for ContractInfo {
    fn get_contract_data(&self) -> Option<&Contract> {
        Some(&self.data)
    }
    fn get_contract_seqno(&self) -> Option<u8> {
        self.seqno
    }
}

#[derive(Debug)]
pub struct StateNode {
    // BRUH, remove me PLEASE; I hate my existence
    // this is here because you cannot do a lookup by value for the key. too bad.
    pub hash: Hash,
    // prev: Option<Hash>,
    pub prev: Weak<StateNode>,
    /// marks that this was detected as a contract
    pub contract: Option<ContractInfo>,
    /// this thing is shared to \<address\> by \<contracts\>
    pub shared: dashmap::DashMap<Address, RwWeaakMap<Hash, ContractNode>>,
    /// this thing has child revisions
    pub leafs: dashmap::DashMap<Hash, Arc<StateNode>>,
}

impl contract_interpreter::ContractInfo for StateNode {
    fn get_contract_data(&self) -> Option<&Contract> {
        self.contract.as_ref().map(|a| &a.data)
    }
    fn get_contract_seqno(&self) -> Option<u8> {
        self.contract.as_ref().and_then(|a| a.seqno)
    }
}

#[derive(Debug)]
pub struct ContractNode {
    pub effect: contract_interpreter::ContractEffect,
    pub latests: RwWeaakMap<Hash, StateNode>,
}

#[derive(Debug)]
pub struct GuardianState<Storage> {
    // todo remove with 1.2
    /// access to storage for looking up previous revision's data (signature+witness), not needed with aqua-protocol v1.2
    pub storage: Storage,
    /// mapping genesis hashes to owning pointers to its [`StateNode`]
    pub genesis_map: dashmap::DashMap<Hash, Arc<StateNode>>,
    /// mapping revision hashes to non-owning pointers to its [`StateNode`]
    pub state_forest: RwWeaakMap<Hash, StateNode>,
    /// mapping finished contract's revision hashes to data about them ([`ContractNode`])
    pub contracts: RwWeaakMap<Hash, ContractNode>,
    /// mapping revision hashes of shared revisions to mappings of user+contract_hash to the [`ContractNode`]
    pub shared_revs: dashmap::DashMap<Hash, RwWeaakMap<(Address, Hash), ContractNode>>,
    /// maps guardians to the users they serve
    ///
    /// when a guardian tries to serve multiple users, its served user is [`POISONED`].
    /// valid only as long as the weak ref exists, must be checked on access
    pub guardian_servitude: dashmap::DashMap<Address, (Address, Weak<ContractNode>)>,
    /// mapping certificates to the owning guardian
    ///
    /// the bytes which are used as a key are CertificateDer bytes.
    pub guardian_identities: RwLock<weak_table::WeakKeyHashMap<Weak<[u8]>, (Address, url::Url)>>,
    pub user_lookup: dashmap::DashMap<Address, RwWeaakMap<Hash, ContractNode>>,
}

/// The address given to conflicting entries
pub const POISONED: Address = Address([0xff; 20]);

#[derive(thiserror::Error, Debug)]
pub enum Error<Storage: guardian_common::storage::Storage> {
    #[error("storage error: {0}")]
    Storage(Storage::Error),
    #[error("previous not in state")]
    PrevNotInState,
    #[error("verify failed: {0:?}")]
    Verifier(flagset::FlagSet<verifier::RevisionIntegrity>),
    #[error("contract-interpreter: {0}")]
    ContractInterpreter(#[from] contract_interpreter::ContractParseError),
    #[error("who are you???")]
    Denied,
}

// mod sealed {
//     pub trait IntoMyError {}
// }
// impl<T: sealed::IntoMyError> From<T> for Error {
//     fn from(_: T) -> Self {
//         Error
//     }
// }
// impl IntoMyError for pkc_api::error::Error {}
// impl IntoMyError for contract_interpreter::ContractParseError {}
// pub type Result<T> = ::std::result::Result<T, Error<Storage>>;

#[derive(Clone)]
struct IterDownTree {
    next: Weak<StateNode>,
}

impl Iterator for IterDownTree {
    type Item = Arc<StateNode>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.upgrade()?;
        self.next = current.prev.clone();
        Some(current)
    }
}
impl From<Weak<StateNode>> for IterDownTree {
    fn from(next: Weak<StateNode>) -> Self {
        IterDownTree { next }
    }
}

impl<S> GuardianState<S> {
    pub fn new(s: S) -> Self {
        Self {
            storage: s,
            genesis_map: Default::default(),
            state_forest: Default::default(),
            contracts: Default::default(),
            shared_revs: Default::default(),
            guardian_identities: Default::default(),
            guardian_servitude: Default::default(),
            user_lookup: Default::default(),
        }
    }
    pub fn get_node(&self, hash: &Hash) -> Option<Arc<StateNode>> {
        self.state_forest.read().get(hash)
    }
    // fn get_branch_iter(&self, hash: &Hash) -> IterDownTree {
    //     self.get_node(hash).as_ref().map(Arc::downgrade).unwrap_or_default().into()
    // }
}

impl<S: Storage> GuardianState<S> {
    fn add_contract_to(&self, contract: (Hash, Arc<ContractNode>), to: (Address, Hash)) {
        let (contract_hash, contract_node) = contract;
        let (addr, page_hash) = to;
        // insert into shared_revs
         eprintln!("Debug write: add contract to shared_revs");
        self.shared_revs
            .entry(page_hash)
            .or_default()
            .write()
            .insert((addr, contract_hash), contract_node.clone());

        if let Some(node) = self.get_node(&page_hash) {
            // add down to root,
            let mut prev = node.prev.clone();
            while let Some(aprev) = prev.upgrade() {
                 eprintln!("Debug write: certverifier change");
                aprev
                    .shared
                    .entry(addr)
                    .or_default()
                    .write()
                    .insert(contract_hash, contract_node.clone());
                prev = aprev.prev.clone();
            }
            // add to self,
            // add to all children
            let mut childstack: Vec<_> = vec![node];
            while let Some(decendent) = childstack.pop() {
                eprintln!("Debug write: add contract to decendent.shared");
                decendent
                    .shared
                    .entry(addr)
                    .or_default()
                    .write()
                    .insert(contract_hash, contract_node.clone());
                if !decendent.leafs.is_empty() {
                    childstack.extend(decendent.leafs.iter().map(|a| a.value().clone()));
                } else {
                     eprintln!("Debug write: add contract to contract_node.latest");
                    // > latest children get listed under latests
                    contract_node
                        .latests
                        .write()
                        .insert(decendent.hash, decendent);
                }
            }
        }
    }

    //for shared add pkc: pkc_api::Pkc
    pub async fn add(&self, hash: Hash, revision: Revision) -> Result<Arc<StateNode>, Error<S>> {
        let prev = match &revision.metadata.previous_verification_hash {
            Some(prev) => {
                let prev_node = self.get_node(prev).ok_or(Error::PrevNotInState)?;
                // todo: verify storage isn't lying to us
                let prev_ref = self.storage.read(*prev).await.map_err(Error::Storage)?;
                Some((prev_node, prev_ref))
            }
            None => None,
        };
        // check if the revision is a genesis
        let is_genesis = prev.is_none();

        let prev_v1_1 = prev.as_ref().map(|(_node, prev)| prev);
        let integrity = verifier::v1_1::revision_integrity(&revision, prev_v1_1);

        let integrity = verifier::v1_1::ignore_absent(integrity);

        if !integrity.is_empty() {
            eprintln!("[{hash}]: integrity error: {integrity:?}");
            return Err(Error::Verifier(integrity));
        }

        if let Some(already_here) = self.get_node(&hash) {
            eprintln!("[{hash}]: duplicate");
            return Ok(already_here);
        }

        let rev_v1_2 = verifier::v1_2::rev_v1_1_to_rev_v1_2(&revision, prev_v1_1, None);

        // create a weak reference to the previous node so that we can reference it
        let prev_weak = prev
            .map(|(prev_node, _)| Arc::downgrade(&prev_node))
            .unwrap_or_default();

        // create contract info if the revision is a contract
        let contract_info = if let Some(res) = Contract::from_revision(&rev_v1_2) {
            let contract = res?;
            let contract_seq = contract.sequence_number(&rev_v1_2);

            // create an iterator down the tree to check if the contract is effective
            let iter = {
                let first_as_once_iter: std::iter::Once<
                    Arc<dyn contract_interpreter::ContractInfo>,
                > = std::iter::once(Arc::new((&contract, contract_seq)));

                let down_iter = IterDownTree::from(prev_weak.clone())
                    .map(|a| -> Arc<dyn contract_interpreter::ContractInfo> { a });
                first_as_once_iter.chain(down_iter)
            };

            let effective = 'effectshit: {
                let Some(effect) = contract_interpreter::is_contract_effective(iter) else {
                    break 'effectshit None;
                };

                // create arc contract_node so that we can reference it and put it into other data structures
                let contract_node = Arc::new(ContractNode {
                    effect: effect.clone(),
                    latests: RwLock::default().into(),
                });
                // add it to the list of effective contracts
                 eprintln!("Debug write: add to list of effective contracts");
                self.contracts.write().insert(hash, contract_node.clone());

                // store the only owned copy away in the info on the revision so it gets deleted when revision gets deleted
                Some(contract_node)
            };

            Some(ContractInfo {
                data: contract,
                seqno: contract_seq,
                effective,
            })
        } else {
            None
        };

        let shared = {
            let map: dashmap::DashMap<Address, RwWeaakMap<Hash, ContractNode>> =
                dashmap::DashMap::default();
            if let Some(ref prev_node) = prev_weak.upgrade() {
                // let prev_shared = prev_node.shared.read();
                // apply all contracts from parent node to new leaf
                for prev_shared_map_ref in prev_node.shared.iter() {
                    // todo! check if contracts should be applied to child as well
                    // map.insert((*user, *contract_hash), contract);
                    eprintln!("Debug read: New applicable contracts");
                    let new_applicable_contracts: WeakValueHashMap<_, _> = prev_shared_map_ref
                        .read()
                        .iter()
                        .filter(|(_contract_hash, _contract)| {
                            // todo! check if contracts should be applied to child as well
                            // for now just default to yes
                            true
                        })
                        .map(|(h, c)| (*h, c))
                        .collect();

                    map.insert(
                        *prev_shared_map_ref.key(),
                        RwLock::new(new_applicable_contracts).into(),
                    );
                }
                //for
            }
            map
        };

        let state_node = Arc::new(StateNode {
            hash,
            prev: prev_weak.clone(),
            leafs: Default::default(),
            contract: contract_info,
            shared,
        });

        // now we need to store the only real arc somewhere to not get deleted at end of scope
        if is_genesis {
            // store in genesis map
            self.genesis_map.insert(hash, state_node.clone());
        } else if let Some(prev_node) = prev_weak.upgrade() {
            // insert ourselves into the previous node's children
            prev_node.leafs.insert(hash, state_node.clone());
        }
        // insert ourself into the state_forest so that we can be found by hash
        eprintln!("Debug write: insert into state_forest");
        self.state_forest.write().insert(hash, state_node.clone());

        // check what we ourselves are shared by (from shared_revs), aka: a contract which shares us existed before us
        if let Some(x) = self.shared_revs.get(&hash) {
            eprintln!("Debug read: check what we share");
            for ((addr, contract_hash), contract_node) in x.read().iter() {
                self.add_contract_to((*contract_hash, contract_node), (*addr, hash));
            }
        }
        
        // find shared latests
        if let Some(ContractInfo {
            effective: Some(contract_node),
            ..
        }) = &state_node.contract
        {
            match &contract_node.effect {
                contract_interpreter::ContractEffect::AccessAgreement((aa, e)) => {
                    use contract_interpreter::AccessAgreementEffects::*;
                    if matches!(e, Granted | Accepted) {
                        for (_, page) in aa.pages.iter() {
                            self.add_contract_to(
                                (hash, contract_node.clone()),
                                (aa.receiver, *page),
                            );
                        }
                        eprintln!("Debug write: contract_node effect Granted/Accepted user lookup");
                        self.user_lookup
                            .entry(aa.receiver)
                            .or_default()
                            .write()
                            .insert(hash, contract_node.clone());
                    }
                    if matches!(e, Offered) {
                        self.add_contract_to(
                            (hash, contract_node.clone()),
                            (aa.receiver, state_node.hash),
                        );
                        eprintln!("Debug write: contract_node effect offered user lookup");
                        self.user_lookup
                            .entry(aa.receiver)
                            .or_default()
                            .write()
                            .insert(hash, contract_node.clone());
                    }
                    if matches!(e, Accepted) {
                        self.add_contract_to(
                            (hash, contract_node.clone()),
                            (aa.sender, state_node.hash),
                        );
                        eprintln!("Debug write: contract_node effect Accepted user lookup");
                        self.user_lookup
                            .entry(aa.sender)
                            .or_default()
                            .write()
                            .insert(hash, contract_node.clone());
                    }
                }
                contract_interpreter::ContractEffect::GuardianServitude((gs, e)) => {
                    use contract_interpreter::GuardianServitudeEffects::*;

                    if matches!(e, Accepted) {
                        self.guardian_servitude.entry(gs.guardian)
                        .and_modify(|addr| {
                            if *addr.0 != gs.user.0 {
                                eprintln!("address collision! guardian {} wants to serve both {} and {}, bad. this must not happen. guardian now serves no one.", gs.guardian, gs.user, addr.0);
                                *addr = (POISONED, Weak::default());
                            }
                        })
                        .or_insert((gs.user, Arc::downgrade(contract_node)));
                    }
                }
                contract_interpreter::ContractEffect::TlsIdentityClaim((tic, e)) => {
                    use contract_interpreter::TlsIdentityClaimEffects::*;
                    if matches!(e, IdentityClaimed) {
                        use weak_table::weak_key_hash_map::Entry::*;
                        eprintln!("Debug write: contract node tls identity claim matches identites");
                        match self.guardian_identities.write().entry(tic.cert.clone()) {
                            Occupied(mut o) => {
                                let (addr, url) = o.get();
                                if *addr != tic.guardian {
                                    eprintln!("certificate collision! certificate [below] is listed by guardians {} and {}. this should never be the case, as such now this cert is for no one.\n{:?}", tic.guardian, addr, tic.cert);
                                    o.insert((POISONED, url.clone()));
                                }
                            }
                            Vacant(v) => {
                                if let Ok(url) = format!("https://{}:{}", tic.host, tic.port).parse() {
                                    v.insert((tic.guardian, url));
                                }
                            }
                        }
                    }
                }
            }
        };

        // possibly remove previous node from contracts we are the latest in now
        {
            let prev_hash_to_be_removed = prev_weak
                .upgrade()
                .and_then(|prev_node| prev_node.leafs.is_empty().then_some(prev_node.hash));

            for refmulti in state_node.shared.iter() {
                eprintln!("Debug read: rm previous from latest");
                for contract in refmulti.read().iter().map(|(_h, arc)| arc) {
                    eprintln!("Debug write: remove delete latest");
                    let mut contract_mut = contract.latests.write();

                    // remove previous node from latests if we are single child
                    if let Some(prev_hash) = prev_hash_to_be_removed {
                        contract_mut.remove(&prev_hash);
                    }

                    contract_mut.insert(hash, state_node.clone());
                }
            }
        }

        Ok(state_node)
    }
    /// removes a node from the data store, though make sure to delete the extracted node as quickly as you can
    pub fn rm(&self, hash: Hash) -> Option<Arc<StateNode>> {
        eprintln!("Debug read: remove function");
        let state_node = self.state_forest.read().get(&hash)?;

        match state_node.prev.upgrade() {
            Some(prev_node) => {
                // remove own node from parent node
                prev_node.leafs.remove(&hash);
                // if we were the only leaf, make parent a latest of all its contracts
                if prev_node.leafs.is_empty() {
                    for refmulti in prev_node.shared.iter() {
                        for (_hash, contract_node) in refmulti.read().iter() {
                            eprintln!("Debug write: remove from contract_node.latest");
                            contract_node
                                .latests
                                .write()
                                .insert(prev_node.hash, prev_node.clone());
                        }
                    }
                }
            }
            None => {
                // remove own node from genesis nodes
                self.genesis_map.remove(&hash)?;
            }
        }

        Some(state_node)
    }

    /// returns all the latest revisions of the given owner that are accessible to the given user
    pub fn get_accessible_latests(
        &self,
        user: Address,
        owner: Address,
    ) -> std::collections::HashSet<Hash> {
        let Some(applicable_contracts) = self.user_lookup.get(&user) else {
            return Default::default();
        };

        let mut set = std::collections::HashSet::new();

        //eprintln!("Debug read: get accessible latest");
        //eprintln!("{:#?}", &state);
        for (contract_hash, contract) in applicable_contracts.read().iter() {
            match &contract.effect {
                ContractEffect::AccessAgreement((aa, e)) => {
                    use contract_interpreter::AccessAgreementEffects::*;
                    if matches!(e, Granted | Accepted) && aa.sender == owner {
                        eprintln!("Debug read: contract if granted or accepted");
                        set.extend(contract.latests.read().keys().copied());
                    }
                    if matches!(e, Offered) && aa.sender == owner {
                        set.insert(*contract_hash);
                    }
                    if matches!(e, Accepted) && aa.receiver == owner {
                        set.insert(*contract_hash);
                    }
                }
                ContractEffect::GuardianServitude(_) => {
                    // nothing
                }
                ContractEffect::TlsIdentityClaim(_) => {
                    // nothing
                } // _ => {
                  //     eprintln!(
                  //         "unhandled contract, skipping while trying to share to {}",
                  //         user
                  //     );
                  // }
            }
        }
        eprintln!("set: {:?}",set);
        set
    }

    pub fn get_rev_accessible(
        &self,
        user: Address,
        hash: Hash,
        owner: Address,
    ) -> Option<Arc<StateNode>> {
        let state_node = self.get_node(&hash)?;
        let applicable_contracts = state_node.shared.get(&user)?;
        eprintln!("Debug read: get rev acccessible");
        for (_contract_hash, contract_node) in applicable_contracts.read().iter() {
            match &contract_node.effect {
                ContractEffect::AccessAgreement((aa, e)) => {
                    use contract_interpreter::AccessAgreementEffects::*;
                    match e {
                        Granted | Accepted if aa.sender == owner => {
                            assert_eq!(aa.receiver, user);
                            // dirty
                            eprintln!("Debug read: DAA Granted / Accepted rev accessible");
                            let rdr = self.state_forest.read();
                            for (_, file) in &aa.pages {
                                if rdr.get(file).is_none() {
                                    continue;
                                }
                            }
                        }
                        Offered if aa.sender == owner => {
                            assert_eq!(aa.receiver, user);
                            // shares itself and previous
                        }
                        Accepted if aa.receiver == owner => {
                            assert_eq!(aa.sender, user);
                            // shares itself and previous
                        }
                        _ => continue,
                    }
                }
                ContractEffect::GuardianServitude(_) => continue,
                ContractEffect::TlsIdentityClaim(_) => continue,
            }
            return Some(state_node.clone());
        }
        None
    }

    pub fn get_accessible_branch(
        &self,
        user: Address,
        hash: Hash,
        owner: Address,
    ) -> Option<Vec<Hash>> {
        let rev = self.get_rev_accessible(user, hash, owner)?;
        Some(
            IterDownTree {
                next: Arc::downgrade(&rev),
            }
            .map(|node| node.hash)
            .collect(),
        )
    }

    pub fn guardian_servitude(&self, guardian: Address) -> Option<Address> {
        self.guardian_servitude.get(&guardian).and_then(|r| {
            let (user, contract) = r.value();
            contract.upgrade().map(|_| *user)
        })
    }

    pub fn guardian_identity(&self, cert_bytes: &[u8]) -> Option<Address> {
        eprintln!("Debug read: Guardian Identity");
        self.guardian_identities.read().get(cert_bytes).map(|a|a.0)
    }
}
