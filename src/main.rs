#![allow(clippy::type_complexity)]

use std::{fmt::Debug, io::Read, net::IpAddr, sync::Arc};

use futures::StreamExt;
use guardian::GuardianState;
use guardian_api::{
    server::{cert_verifier::CertVerifier, ServerInfo},
    ApiClient, ApiHandler, ApiServer,
};
use guardian_common::{custom_types::Hash, signing::Signer, storage::Storage};
use pkc_api::storage::RevContext;
use webpki::types::CertificateDer;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let private_key: guardian_common::signing::SimpleSigner = std::env::var("PRIVATE_KEY")
        .expect("no private key")
        .parse()
        .expect("failed to parse private key");
    let pkc_url: url::Url = std::env::var("PKC_URL")
        .expect("no pkc url")
        .parse()
        .expect("not a valid url");
    let admin_user: ethaddr::Address = std::env::var("ADMIN_USER")
        .expect("no admin user")
        .parse()
        .expect("failed to parse ADMIN_USER");
    let host: IpAddr = std::env::var("HOST")
        .expect("no host")
        .parse()
        .expect("failed to parse host");
    let port: u16 = std::env::var("PORT")
        .expect("no port")
        .parse()
        .expect("failed to parse PORT");

    // check if the hostname in the tls certificate is the same as the hostname in .env
    // this is a scope, so no one gets confused by all the variables declared here.
    {
        // Try open the PEM file
        // make new one if not found
        match std::fs::File::open("identity.pem") {
            Ok(_file) => {}
            _ => {
                println!("failed to open identity.pem. would you like to generate it? [y/n]");
                let mut ans = String::new();
                std::io::stdin()
                    .read_line(&mut ans)
                    .expect("Failed to read line");
                match ans.trim().to_lowercase().starts_with('y') {
                    true => {
                        println!("writing privatekey and certificate.");
                        guardian::certificate_generation::gen_identity();
                    }
                    false => {
                        panic!("aborting guardian setup");
                    }
                }
            }
        }
        let pem_file: std::fs::File = std::fs::File::open("identity.pem").unwrap();

        let reader = std::io::BufReader::new(pem_file);

        // Read the PEM content
        let cert_pem = reader
            .bytes()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .collect::<Vec<u8>>();

        // Create an X509 object from the PEM data, so we can read fields in the certificate
        let cert = openssl::x509::X509::from_pem(&cert_pem)
            .expect("failed to get certificate from identity.pem");
        let altnames = cert
            .subject_alt_names()
            .expect("certificate in identity.pem has not hostname");

        let mut good_hostname = false;
        let mut host_ip;
        for altname in altnames {
            match host {
                IpAddr::V4(addr) => host_ip = addr.octets().to_vec(),
                IpAddr::V6(addr) => host_ip = addr.octets().to_vec(),
            };

            // check if any of the hostnames correspond to the one in .env
            if let Some(ip) = altname.ipaddress() {
                if ip == host_ip {
                    good_hostname = true;
                }
            }
        }
        if !good_hostname {
            println!("The guardian has a Certificate, but its hostname does not match yours. \nDo you want to generate a new one (overwrite current)? [y/n]");
            let mut ans = String::new();
            std::io::stdin()
                .read_line(&mut ans)
                .expect("Failed to read line");
            match ans.trim().to_lowercase().starts_with('y') {
                true => {
                    println!("overwriting certificate.");
                    guardian::certificate_generation::gen_identity();
                }
                false => {
                    panic!("aborting guardian setup");
                }
            }
        }
    }

    let client = siwe_oidc_auth::login(&private_key, &pkc_url).await;
    let pkc = pkc_api::Pkc::new_with_options(chrono::Utc::now().naive_utc(), pkc_url, client);

    let latests = pkc.list().await.expect("couldn't get all pages");

    let fire = Campfire::new(pkc.clone());
    let genesi = fire
        .build(latests)
        .await
        .expect("failed to build revision forest");
    let afire = ::std::sync::Arc::new(fire);
    for err in afire.clone().burn(genesi).await {
        eprintln!("error while burning forest: {err}");
    }
    let Campfire { state, .. } =
        ::std::sync::Arc::try_unwrap(afire).expect("unexpected error in processing state entity");

    eprintln!("{:#?}", &state);

    if let Some(user) = state.guardian_servitude(private_key.identity().into()) {
        if user != admin_user {
            panic!("Servitude INVALID: signed by invalid user");
        }

        println!("{admin_user} has accepted servitude");
    } else {
        println!("Admin has not accepted my request yet, sending a new one. Please sign the Servitude contract in the PKC with the Wallet key used to deploy the PKC (authoritative key = admin).");

        let gs = guardian::contract_generation::make_guardian_servitude(admin_user, &private_key);

        let genesis_hash = gs.first().unwrap().metadata.verification_hash;
        for thing2 in gs {
            pkc.store(
                thing2,
                RevContext {
                    namespace: 0,
                    name: format!(
                        "Servitude:{}",
                        ethaddr::Address::from(private_key.identity())
                    ),
                    genesis_hash,
                    domain_id: ethaddr::Address::from(private_key.identity()).to_string(),
                },
            )
            .await
            .expect("failed to store");
        }
    }

    // create and push a TLS Certificate in the PKC
    // let key_pair = guardian_api::read_keys("identity").expect("failed to read tls private key");
    let cert =
        guardian::certificate_generation::read_certs("identity").expect("failed to read tls cert");
    let cert = cert.first().expect("tls cert not here");

    // dbg!(&cert.der()[..]);
    if let Some(guardian) = state.guardian_identity(cert) {
        println!("The Guardian {guardian} already has the self-signed TLS Certificate!");
    } else {
        println!("The Guardian doesn't have a TLS Certificate. \n Creating and signing one...");

        // let key_pair =
        //     rcgen::KeyPair::try_from(&key_pair).expect("failed to make tls into rcgen keypair");
        // let key_pair_pem = key_pair.serialize_pem();

        // let (cert, tls_cert) =
        //     guardian::contract_generation::make_guardian_cert(key_pair, host, port, &private_key)
        //         .unwrap();

        let tls_cert = guardian::contract_generation::make_tls_cert_contract(
            host,
            port,
            &private_key,
            cert.to_owned(),
        )
        .unwrap();

        let genesis_hash = tls_cert.first().unwrap().metadata.verification_hash;
        for thing2 in tls_cert {
            pkc.store(
                thing2,
                RevContext {
                    namespace: 0,
                    name: format!("TlsCert:{}", ethaddr::Address::from(private_key.identity())),
                    genesis_hash,
                    domain_id: ethaddr::Address::from(private_key.identity()).to_string(),
                },
            )
            .await
            .expect("failed to store TLS Certificate");
        }
    }

    let server_certificate =
        guardian::certificate_generation::read_certs("identity").unwrap()[0].to_owned();
    let server_keypair = guardian::certificate_generation::read_keys("identity").unwrap();

    eprintln!("Debug read: client_certs");
    let client_certs: Vec<CertificateDer<'static>> = state
        .guardian_identities
        .read()
        .keys()
        .map(|slice| CertificateDer::from(slice.to_vec()))
        .collect();

    let trust: Vec<_> = client_certs
        .iter()
        .map(|cert| webpki::anchor_from_trusted_cert(cert))
        .collect::<Result<Vec<_>, _>>()
        .expect("certificate verification failed")
        .into_iter()
        .map(|a| a.to_owned())
        .collect();

    let trusted = CertVerifier::new();
    trusted.set(trust.into());

    let conn = ServerInfo {
        addr: (host, port).into(),
        trusted: trusted.clone(),
        cert_chain: vec![server_certificate.clone()],
        key_der: server_keypair,
    };

    let astate = std::sync::Arc::new(state);
    let bstate = astate.clone();

    let get_handler = move |info: &[webpki::types::CertificateDer<'static>]| {
        let cert = info
            .first()
            .expect("unexpected error in certificate handler")
            .to_owned();
        let bstate = bstate.clone();
        async move {
            Handler {
                state: bstate.clone(),
                cert,
                admin_user,
            }
        }
    };
    // run server
    tokio::spawn(guardian_api::server::GuardianServer::run(conn, get_handler).unwrap());

    let trusted2 = trusted.clone();
    let identity = reqwest::Identity::from_pem(&std::fs::read("identity.pem").unwrap())
        .expect("identity not found help");
    let bstate = astate.clone();
    let pkc2 = pkc.clone();
    let run_client = move |cert: Arc<[u8]>, url: url::Url| async move {
        let cert_ta;

        let xyz: CertificateDer<'static> = CertificateDer::from(cert.to_vec());
        match webpki::anchor_from_trusted_cert(&xyz) {
            Ok(certta) => {
                cert_ta = certta.clone();
                trusted2.change(|ta| {
                    eprintln!("adding certificate");
                    let mut ta: Vec<webpki::types::TrustAnchor<'static>> = ta.to_vec();
                    ta.push(certta.to_owned());
                    ta.into()
                });
            }
            _ => return,
        }

        let x = Arc::downgrade(&cert);
        drop(cert);

        'untrust: while let Some(cert) = x.upgrade() {
            let client =
                guardian_api::client::GuardianClient::<RevContext>::new(guardian_api::ConnInfo {
                    url: url.clone(),
                    cert: reqwest::Certificate::from_der(&cert).unwrap(),
                    identity: identity.clone(),
                })
                .await
                .expect("failed to create client");
            'fail: loop {
                let Ok(latests) = client.list().await else {
                    break 'fail;
                };
                for latest in latests {
                    if bstate.get_node(&latest).is_some() {
                        continue;
                    }
                    let Ok(branch) = client.get_branch(latest).await else {
                        break 'fail;
                    };
                    let mut x = vec![];
                    for branch_hash in branch.hashes {
                        if bstate.get_node(&branch_hash).is_none() {
                            x.push(branch_hash);
                        } else {
                            break;
                        }
                    }
                    for branch_hash in x.into_iter().rev() {
                        let Ok(rev) = client.get_revision(branch_hash).await else {
                            break 'fail;
                        };
                        let prev_rev =
                            if let Some(prev_hash) = rev.metadata.previous_verification_hash {
                                let Ok(prev_rev) = pkc2.read(prev_hash).await else {
                                    break 'fail;
                                };
                                Some(prev_rev)
                            } else {
                                None
                            };
                        let integrity = verifier::v1_1::revision_integrity_ignore_absent(
                            &rev,
                            prev_rev.as_ref(),
                        );
                        if integrity.bits() != 0 {
                            eprintln!(
                                "integrity verification failed!\n[{branch_hash}]: {integrity:?}, {rev:#?}"
                            );
                            break 'untrust;
                        }
                        let Ok(_) = pkc2.store(rev, branch.metadata.clone()).await else {
                            break 'fail;
                        };
                    }
                }
                //Set the time for the update intervals in milliseconds
                tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
            }
            //Set the time for the update intervals in milliseconds
            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        }

        trusted.change(|ta| {
            ta.iter()
                .filter_map(|a| (a != &cert_ta).then_some(a.to_owned()))
                .collect::<Vec<_>>()
                .into()
        });
    };
    let ethaddr = ethaddr::Address::from(private_key.identity());
    eprintln!("Debug read: spawn/run clients");
    for (cert, (addr, url)) in astate.guardian_identities.read().iter() {
        //Check if not for yourself, if true spawn a new client
        if *addr != ethaddr {
            tokio::spawn(run_client.clone()(cert, url.clone()));
        }
    }

    let pkc2 = pkc.clone();
    pkc.update_handler(move |hash, kind| {
        dbg!(hash);
        let pkc2 = pkc2.clone();
        let run_client = run_client.clone();
        let astate = astate.clone();
        tokio::spawn(async move {
            let del = "delete";
            if kind != del {
                eprintln!("kind: {:?}",kind);
                let branch = pkc2.get_branch(hash).await.unwrap();
                let hashes = branch.hashes;
                let mut have = None;
                dbg!(&hashes);
                for hash in &hashes {
                    if astate.get_node(hash).is_some() {
                        have = Some(*hash);
                        break;
                    }
                }
                dbg!(&have);
                for need in hashes.into_iter().rev() {
                    if let Some(has) = have {
                        if need == has {
                            have = None;
                        }
                        continue;
                    }
                    eprintln!("Debug read: Update Handler");
                    let x = pkc2
                        .read(need)
                        .await
                        .expect("failed getting promised branch revision");
                    let added = astate
                        .add(need, x)
                        .await
                        .expect("failed adding promised branch rev");
                    if let Some(guardian::ContractInfo {effective: Some(eff), .. }) = &added.contract {
                        let guardian::ContractNode{ effect, .. } = eff.as_ref();
                        eprintln!("effect: {:?}",effect);
                        match effect {
                            contract_interpreter::ContractEffect::AccessAgreement((_aa, _e)) => {},
                            contract_interpreter::ContractEffect::GuardianServitude((_gs, _e)) => {},
                            contract_interpreter::ContractEffect::TlsIdentityClaim((tic, e)) => {
                                match e {
                                    contract_interpreter::TlsIdentityClaimEffects::IdentityClaimed => {
                                        if tic.guardian != ethaddr {
                                            eprintln!("Debug read: Identity Claim");
                                            if let Some((_addr, url)) = astate.guardian_identities.read().get(&tic.cert) {
                                                tokio::spawn(run_client.clone()(tic.cert.clone(), url.clone()));
                                            }
                                        }
                                    },
                                }
                            },
                        }
                    }
                }
            } else if let Some(mut sn) = astate.get_node(&hash) {
                loop {
                    let sn2 = sn;
                    if sn2.leafs.is_empty() {
                        astate.rm(sn2.hash);
                    } else {
                        break;
                    }
                    if let Some(sns) = sn2.prev.upgrade() {
                        sn = sns;
                    } else {
                        break;
                    }
                }
            };
            eprintln!("{:#?}", &astate);
        });
    })
    .await
    .expect("update handler failed");
}

#[derive(Debug)]
struct Campfire<Storage> {
    forest: dashmap::DashMap<Hash, Vec<Hash>>,
    storage: Storage,
    state: GuardianState<Storage>,
}

impl<S: Clone + Storage + Sync + Send + 'static> Campfire<S> {
    pub fn new(storage: S) -> Self {
        Self {
            forest: dashmap::DashMap::new(),
            state: GuardianState::<S>::new(storage.clone()),
            storage,
        }
    }
    /// build the forest
    async fn build(&self, latests: Vec<Hash>) -> Result<dashmap::DashSet<Hash>, S::Error> {
        let genesis_set = dashmap::DashSet::new();
        let genesis_set_ref = &genesis_set;

        let requests = latests.into_iter().map(|latest| async move {
            let mut iter = self.storage.get_branch(latest).await?.hashes.into_iter();
            let mut next = iter
                .next()
                .expect("unexpected error in state forest build iterator");

            // insert leaf
            self.forest.entry(next).or_default();

            // insert path to bottom
            for prev in iter {
                match self.forest.entry(prev) {
                    dashmap::mapref::entry::Entry::Occupied(mut o) => {
                        o.get_mut().push(next);
                    }
                    dashmap::mapref::entry::Entry::Vacant(v) => {
                        v.insert(vec![next]);
                    }
                }
                next = prev;
            }

            genesis_set_ref.insert(next);

            Ok(())
        });
        let mut req_stream = futures::stream::FuturesUnordered::from_iter(requests);

        while let Some(answer) = req_stream.next().await {
            answer?;
        }
        drop(req_stream);

        Ok(genesis_set)
    }

    /// light the fire
    async fn burn(
        self: ::std::sync::Arc<Self>,
        genesis_set: dashmap::DashSet<Hash>,
    ) -> impl Iterator<Item = guardian::Error<S>>
    where
        <S as guardian_common::storage::Storage>::Error: Send + Sync,
    {
        let (sendr, recvr) = crossbeam::channel::unbounded();
        for genesis_hash in genesis_set {
            tokio::spawn(self.clone().burn_elem(genesis_hash, sendr.clone()));
        }
        drop(sendr);
        recvr.into_iter()
    }
    #[allow(clippy::manual_async_fn)]
    fn burn_elem(
        self: ::std::sync::Arc<Self>,
        hash: Hash,
        sendr: crossbeam::channel::Sender<guardian::Error<S>>,
    ) -> impl futures::Future<Output = ()> + Send + Sync
    where
        <S as guardian_common::storage::Storage>::Error: Send + Sync,
    {
        eprintln!("Debug read: burn element");
        async move {
            if let Err(e) = async {
                let rev = self
                    .storage
                    .read(hash)
                    .await
                    .map_err(guardian::Error::Storage)?;
                let _handle = self.state.add(hash, rev).await?;
                Ok(())
            }
            .await
            {
                sendr.send(e).ok();
            }
            for leaf in self
                .forest
                .remove(&hash)
                .into_iter()
                .flat_map(|(_hash, v)| v.into_iter())
            {
                eprintln!("starting {leaf}");
                tokio::spawn(self.clone().burn_elem(leaf, sendr.clone()));
            }
        }
    }
}

#[derive(Clone)]
struct Handler<S> {
    state: Arc<GuardianState<S>>,
    cert: CertificateDer<'static>,
    admin_user: ethaddr::Address,
}
impl<S: guardian_common::storage::Storage> Handler<S> {
    fn get_addr(&self) -> Result<ethaddr::Address, guardian::Error<S>> {
        let Some((guardian_addr, _)) = self
            .state
            .guardian_identities
            .read()
            .get(&self.cert)
            .cloned()
        else {
            return Err(guardian::Error::Denied);
        };
        self.state
            .guardian_servitude
            .get(&guardian_addr)
            .ok_or(guardian::Error::Denied)
            .map(|a| a.0)
    }
}
impl<S: Storage + Debug + Send + Sync> ApiHandler for Handler<S> {
    type Error = guardian::Error<S>;
    type Context = S::Context;

    ///lists all hashes avalilabe for remote user
    async fn list(&self) -> Result<std::collections::HashSet<Hash>, Self::Error> {
        let user = self.get_addr()?;
        let hashes = self.state.get_accessible_latests(user, self.admin_user);
        Ok(hashes)
    }
    /// returns hashes of a branch if the branch is available to the remote user
    async fn get_branch(
        &self,
        hash: Hash,
    ) -> Result<guardian_common::prelude::Branch<Self::Context>, Self::Error> {
        let user = self.get_addr()?;
        let Some(hashes) = self
            .state
            .get_accessible_branch(user, hash, self.admin_user)
        else {
            return Err(Self::Error::Denied);
        };
        let context = self
            .state
            .storage
            .get_context(hash)
            .await
            .map_err(guardian::Error::Storage)?;
        let branch = guardian_common::custom_types::Branch {
            metadata: context,
            hashes,
        };
        Ok(branch)
    }
    /// returns revision if it is available to the remote user
    async fn get_revision(
        &self,
        hash: Hash,
    ) -> Result<guardian_common::prelude::Revision, Self::Error> {
        let user = self.get_addr()?;
        if self
            .state
            .get_rev_accessible(user, hash, self.admin_user)
            .is_some()
        {
            eprintln!("Debug read: get_revision");
            let rev = self
                .state
                .storage
                .read(hash)
                .await
                .map_err(guardian::Error::Storage)?;
            Ok(rev)
        } else {
            Err(Self::Error::Denied)
        }
    }
}
