use std::{collections::HashSet, net::SocketAddr};

use guardian_api::{
    client::GuardianClient,
    server::{cert_verifier::CertVerifier, GuardianServer, ServerInfo},
    ApiClient, ApiHandler, ApiServer, ConnInfo,
};
use guardian_common::prelude::Revision;
use pkc_api::storage::RevContext;
use rcgen::{Certificate, KeyPair};

fn gen_keys<HostnamesList: Into<Vec<String>>>(
    _name: &str,
    hostnames: HostnamesList,
) -> Result<(Certificate, KeyPair), Box<dyn std::error::Error + Send + Sync>> {
    let rcgen::CertifiedKey {
        cert: client_cert,
        key_pair: client_keypair,
    } = rcgen::generate_simple_self_signed(hostnames)?;
    /*std::fs::write(format!("{_name}.pem"), client_keypair.serialize_pem())?;
    std::fs::write(format!("{_name}_cert.pem"), client_cert.pem())?;
    std::fs::write(
        format!("{_name}_curl_cert.perm"),
        format!("{}{}", client_cert.pem(), client_keypair.serialize_pem()),
    )?;*/

    Ok((client_cert, client_keypair))
}

#[tokio::test]
async fn talk() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //init server data
    println!("generating keys");
    let (server_certificate, server_keypair) = gen_keys("server", &["localhost".to_string()])?;
    let server_key_der =
        rustls::pki_types::PrivateKeyDer::try_from(server_keypair.serialize_der())?;

    //init client data
    let (client_a_cert, client_a_keypair) = gen_keys("client_a", &[])?;

    let cert_verifier = CertVerifier::new();

    let info = ServerInfo {
        addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
        trusted: cert_verifier.clone(),
        cert_chain: vec![server_certificate.der().clone()],
        key_der: server_key_der,
    };
    let trusted_client = vec![webpki::anchor_from_trusted_cert(client_a_cert.der())
        .unwrap()
        .to_owned()]
    .into();
    cert_verifier.set(trusted_client);

    //create API Handler
    #[derive(Clone)]
    struct Handler;
    impl ApiHandler for Handler {
        type Error = std::convert::Infallible;
        type Context = RevContext;
        async fn list(
            &self,
        ) -> Result<std::collections::HashSet<guardian_common::prelude::Hash>, Self::Error>
        {
            println!("hey, i'm listing everything");
            let mut set = HashSet::default();
            set.insert("dead3582839c6b2d616fbb96057f6665a68bbe6ec755358822b4329f51e279a9c9fb1b4b466fc8ca3e9c335aacbb759e5977e6a95acfd669e2c40033c08ea40f".parse().unwrap());
            Ok(set)
        }
        async fn get_branch(
            &self,
            hash: guardian_common::prelude::Hash,
        ) -> Result<guardian_common::custom_types::Branch<RevContext>, Self::Error> {
            println!("getting branches");
            Ok(guardian_common::custom_types::Branch {hashes:vec![hash,"bed53582839c6b2d616fbb96057f5555a68bbe6ec755358822b4329f51e279a9c9fb1b4b466fc8ca3e9c335aacbb759e5977e6a95acfd669e2c40033c08ea40f".parse().unwrap()], metadata: RevContext { namespace: 0, name: "Main_Page".to_string(), genesis_hash: Default::default(), domain_id: "domain".to_string() } })
        }
        async fn get_revision(
            &self,
            _hash: guardian_common::prelude::Hash,
        ) -> Result<Revision, Self::Error> {
            println!("I'm giving you stuff, because ...");
            //std::io::stdout().flush();
            Ok(Revision::default())
        }
    }

    let server_creation_state = GuardianServer::run(info, |_client_info| async {
        println!("generating server handler");
        Handler
    })
    .unwrap();
    tokio::spawn(server_creation_state);

    //client stuff
    let client_identity = reqwest::Identity::from_pem(
        format!(
            "{}{}",
            client_a_cert.pem(),
            client_a_keypair.serialize_pem()
        )
        .as_bytes(),
    )?;

    println!("creating client");
    let client = GuardianClient::<RevContext>::new(ConnInfo {
        url: "https://localhost:3000".parse().unwrap(),
        cert: reqwest::Certificate::from_der(server_certificate.der()).unwrap(),
        identity: client_identity,
    })
    .await
    .unwrap();

    let list_response = client.list().await.unwrap();
    dbg!(&list_response);
    let list_hash = list_response.into_iter().next().unwrap();
    let get_branch_response = client.get_branch(list_hash).await.unwrap();
    dbg!(get_branch_response);
    let get_revision_response = client.get_revision("beef5678901c6b2d616fbb96057f6665a68bbe6ec755358822b4329f51e279a9c9fb1b4b466fc8ca3e9c335aacbb759e5977e6a95acfd669e2c40033c08ea40f".parse().unwrap()).await.unwrap();
    dbg!(get_revision_response);
    //client.get_revision("beef5678901c6b2d616fbb96057f6665a68bbe6ec755359822b4329f51e279a9c9fb1b4b466fc8ca3e9c335aacbb759e5977e6a95acfd669e2c40033c08ea40f".parse().unwrap()).await.expect_err("You're not supposed to parse this");
    Ok(())
}
