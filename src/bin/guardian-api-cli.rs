use std::{
    collections::HashSet,
    io::{self},
    net::SocketAddr,
};

use clap::{Parser, Subcommand};
use guardian_api::{
    server::{cert_verifier::CertVerifier, GuardianServer, ServerInfo},
    ApiClient, ApiHandler, ApiServer,
};
use guardian_common::{custom_types::*, storage::Storage};
use pkc_api::storage::RevContext;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    StartServer {/* port and IP address */},
    ClientSync { server_ip: String },
    GenKeys {}, // only one IP for now
}

// function to generate a private key
// fn gen_identity<HostnamesList: Into<Vec<String>>>(hostnames: HostnamesList) {
//     let rcgen::CertifiedKey { cert, key_pair } =
// rcgen::generate_simple_self_signed(hostnames).expect("failed to generate key and cert");
//     std::fs::write(
//         "identity.pem",
//         format!("{}{}", cert.pem(), key_pair.serialize_pem()),
//     )
//     .expect("failed to write key and cert to file");
// }

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let _private_key: guardian_common::signing::SimpleSigner = std::env::var("PRIVATE_KEY")
        .expect("no private key")
        .parse()
        .expect("failed to parse private key");
    let pkc_url: url::Url = std::env::var("PKC_URL")
        .expect("no pkc url")
        .parse()
        .expect("not a valid url");
    let _admin_user: ethaddr::Address = std::env::var("ADMIN_USER")
        .expect("no admin user")
        .parse()
        .expect("failed to parse ADMIN_USER");
    let _host: std::net::IpAddr = std::env::var("HOST")
        .expect("no host")
        .parse()
        .expect("failed to parse host");
    let _port: u16 = std::env::var("PORT")
        .expect("no port")
        .parse()
        .expect("failed to parse PORT");

    let pkc = pkc_api::Pkc::new(pkc_url).unwrap();
    dotenv::dotenv().ok();

    let args = Cli::parse();

    match args.command {
        Commands::StartServer {} => {
            let server_keypair = guardian::certificate_generation::read_keys("identity").unwrap();

            // read our own cert from file
            let server_certificate =
                guardian::certificate_generation::read_certs("identity").unwrap()[0].to_owned();

            // read certificates of authorized client from file
            let client_certificates =
                guardian::certificate_generation::read_certs("clients").unwrap();

            let cert_verifier = CertVerifier::new();
            // setup server
            let info = ServerInfo {
                addr: SocketAddr::from(([0, 0, 0, 0], 3000)),
                trusted: cert_verifier.clone(),
                cert_chain: vec![server_certificate.clone()],
                key_der: server_keypair,
            };

            // add all client certs from file
            let trusted_clients: std::sync::Arc<[webpki::types::TrustAnchor<'_>]> =
                client_certificates
                    .iter()
                    .map(|cert| webpki::anchor_from_trusted_cert(cert).unwrap().to_owned())
                    .collect();

            cert_verifier.set(trusted_clients);

            //create API Handler
            #[derive(Clone)]
            struct Handler(pkc_api::Pkc);
            impl ApiHandler for Handler {
                type Error = pkc_api::error::Error;
                type Context = RevContext;
                async fn list(&self) -> std::result::Result<HashSet<Hash>, pkc_api::error::Error> {
                    let hashes = self.0.list().await.expect("failed to list all revisions");
                    let hashset = HashSet::from_iter(hashes);
                    Ok(hashset)
                }
                async fn get_branch(
                    &self,
                    hash: guardian_common::prelude::Hash,
                ) -> Result<Branch<RevContext>, pkc_api::error::Error> {
                    let res = self.0.get_branch(hash).await.expect("failed to get branch");
                    Ok(res)
                }
                async fn get_revision(
                    &self,
                    hash: guardian_common::prelude::Hash,
                ) -> Result<Revision, pkc_api::error::Error> {
                    let rev = self.0.read(hash).await.expect("failed to read revision");
                    Ok(rev)
                }
            }

            // run the server
            GuardianServer::run(info, move |_| {
                let handler = Handler(pkc.clone());
                async move {
                    println!("generating server handler");
                    handler
                }
            })
            .unwrap()
            .await
            .expect("server error");
        }
        Commands::ClientSync { server_ip } => {
            let server_certificate =
                guardian::certificate_generation::read_certs("server").unwrap()[0].to_owned(); // hardcoded server name for now

            let url: url::Url = server_ip.parse().expect("not a valid url");

            let client: guardian_api::client::GuardianClient<RevContext> =
                guardian_api::client::GuardianClient::<RevContext>::new(guardian_api::ConnInfo {
                    url,
                    cert: reqwest::Certificate::from_der(&server_certificate).unwrap(),
                    identity: reqwest::Identity::from_pem(&std::fs::read("identity.pem").unwrap())
                        .unwrap(),
                })
                .await
                .unwrap();

            loop {
                sync(&client, &pkc).await.unwrap();

                // sleep for 10 seconds after each sync
                // #TODO: consider not hardcoding sleep duration
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
        Commands::GenKeys {} => {
            // #TODO: consider generating certs hSere as well
            if std::path::Path::new("identity.pem").exists() {
                println!(
                    "Guardian alreay has private key. Do you really want to overwrite it? [y/n]"
                );
                let mut ans = String::new();
                io::stdin()
                    .read_line(&mut ans)
                    .expect("Failed to read line");
                match ans.trim().to_lowercase().starts_with('y') {
                    true => {
                        println!("overwriting certificate.");
                        guardian::certificate_generation::gen_identity();
                    }
                    false => {
                        println!("aborted operation");
                    }
                }
            } else {
                println!("writing private key.");
                guardian::certificate_generation::gen_identity();
            }
        }
    }
}

async fn sync<Client: guardian_api::ApiClient, Pkc: Storage<Context = Client::Context>>(
    client: &Client,
    pkc: &Pkc,
) -> Result<(), Client::Error>
where
    Client::Context: Clone,
{
    // our latest revs
    let our_revs = pkc.list().await.expect("failed to list in sync");
    // let our_revs = vec![];

    // other guardians latest refs
    let foreign_revs = client.list().await?;

    // check if any of the foreign hashes are not in our pkc
    for rev_hash in foreign_revs {
        if !our_revs.contains(&rev_hash) {
            // put foreign branch in pkc

            let foreign_branch = client.get_branch(rev_hash).await.unwrap();
            let mut to_get: Vec<Hash> = Vec::new();
            let mut context = foreign_branch.metadata; // foreign context if not in our pkc
            for hash in foreign_branch.hashes {
                if our_revs.contains(&hash) {
                    // the branch exists in our pkc, so we use its context, not the foreign one
                    context = pkc.get_context(hash).await.unwrap();
                    break;
                } else {
                    to_get.push(hash);
                }
            }

            for hash in to_get.into_iter().rev() {
                pkc.store(client.get_revision(hash).await.unwrap(), context.clone())
                    .await
                    .unwrap();
                dbg!("stored to pkc");
            }
        }
    }

    Ok(())
}
