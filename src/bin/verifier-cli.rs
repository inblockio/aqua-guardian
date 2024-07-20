use std::collections::HashMap;

use clap::{Parser, Subcommand};
use guardian_common::custom_types::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The url of the server, e.g. <https://pkc.inblock.io>
    #[arg(short, long, default_value = "http://localhost:9352")]
    server: reqwest::Url,
    /// OAuth2 access token to access the API
    #[arg(short, long)]
    token: Option<String>,
    /// example/testing key: `72c7193b5776ba92c78fa31143d285317cda41a8e22e2ef2ac5379b8053e6d48`
    #[arg(short, long)]
    private_key: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    VerifyRevision { hash: String },
    VerifyAll {},
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let client = match args.private_key {
        Some(s) => {
            let bytes: [u8; 32] = hex::decode(s)
                .expect("private key not plain hex")
                .try_into()
                .expect("incorrect private key length");
            siwe_oidc_auth::login(
                guardian_common::signing::SimpleSigner::try_from(bytes)
                    .expect("not a valid private key"),
                &args.server,
            )
            .await
        }
        None => reqwest::Client::new(),
    };
    let pkc =
        pkc_api::Pkc::new_with_options(chrono::Utc::now().naive_utc(), args.server.clone(), client);
    match args.command {
        Commands::VerifyRevision { hash } => {
            let hash = hash.parse().expect("given hash could not be parsed");
            let revision: Revision = pkc.da_get_revision(hash).await.unwrap();
            let prev;
            let mut prev_revision: Option<&Revision> = None;
            if let Some(prev_hash) = revision.metadata.previous_verification_hash {
                prev = pkc.da_get_revision(prev_hash).await.unwrap();
                prev_revision = Some(&prev);
            }

            let a = verifier::v1_1::revision_integrity(&revision, prev_revision);

            println!("flags: {:#?}", a);
        }
        Commands::VerifyAll {} => {
            let pkc_integrity = verifier::v1_1::verify_all(pkc).await;

            println!("{:#?}", pkc_integrity);

            let pkc_integrity_pretty: HashMap<_, _> = pkc_integrity
                .into_iter()
                .map(|(x, y)| (x, verifier::v1_1::ignore_absent(y)))
                .collect();

            let content = serde_json::ser::to_string_pretty(&pkc_integrity_pretty).unwrap();
            std::fs::write("logfile.json", content).unwrap();
        }
    }
}
