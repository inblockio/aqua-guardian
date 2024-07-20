use clap::{Parser, Subcommand};
use guardian_common::storage::Storage;
use pkc_api::mediawiki::allpages::PageInfo;


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
    /// timestamp: YYYYMMDDMMSS
    /// deleted: boolean
    /// example: ./pkc-api-cli get-recent-changes 20240629091000 true
    /*
    GetRecentChanges {
        timestamp: String,
        deleted: bool,
    },*/
    GetAllPages {
        // todo
    },
    /*
    GetAllRevisions {
        page_title: String,
    },
    GetRevisionHash {
        rev_id: Option<usize>,
    },
    GetAquaChain {
        genesis_hash: String,
    },*/
    GetLatestRevisionHash {
        page_name: String,
    },
    Read {
        hash: String,
    },
    List {},
    GetBranch {
        latest_revision_hash: String,
    },
    // PushAquaChain {
    //     file: std::path::PathBuf,
    // },
    /*
    PushRevision {
        file: std::path::PathBuf,
    },
    PushRevisionBypass {
        file: std::path::PathBuf,
    },*/
    /*RecentChanges {
        /// example: "2024-06-06T12:10:00"
        ///
        /// format: "%Y-%m-%dT%H:%M:%S%.f"
        ///
        time: chrono::NaiveDateTime,
    },*/
    //Update {},
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
        Commands::GetAllPages {} => {
            let pages: Vec<PageInfo> = pkc.mw_allpages().await.expect("failed to get pages");
            println!("pages: {:#?}", pages);
        }
        /*
        Commands::GetAllRevisionIds { /* page */ } => {
            let rev_ids: Vec<usize> = pkc.da_get_page_all_revs("Main_Page").await.expect("failed to get revisions");
            println!("rev ids are: {:?}", rev_ids)
        },

        Commands::GetRevisionHash { rev_id } => {
            // let rev_id: usize = 9;
           let revision_hash: Hash = pkc.revision_hash_for_rev_id(rev_id.unwrap()).await.expect("failed to get hash of revision");
           println!("revision hash for rev {} = {:?}", rev_id.unwrap(), revision_hash);
        }

        Commands::GetAquaChain { genesis_hash } => {

            let aqua_chain = pkc.chain_from_genesis_hash(genesis_hash.parse().unwrap()).await.expect("failed to get aqua chain");

            println!("{}: {:?}", genesis_hash, aqua_chain.pages[0]);
        }*/
        Commands::GetLatestRevisionHash { page_name } => {
            let latest_revision_hash = pkc
                .latest_hash_for_title(&page_name)
                .await
                .expect("failed to get latest revision hash");

            println!("latest_revision_hash: {:?}", latest_revision_hash);
        }
        Commands::Read { hash } => {
            let rev = pkc
                .read(hash.parse().unwrap())
                .await
                .expect("failed to read revision");
            println!("revision: {:?}", rev);
        }
        Commands::GetBranch {
            latest_revision_hash,
        } => {
            let branch = pkc
                .get_branch(latest_revision_hash.parse().unwrap())
                .await
                .expect("failed to get branch from last_rev");

            println!("branch(with context): {:?}", branch);
        }

        // this does not work at all.
        // Commands::PushAquaChain { file } => {
        //     // todo!("implement the MW VerifiedImport call");

        //     let thing: pkc_api::OfflineData = serde_json::from_reader(std::fs::File::open(file).unwrap()).unwrap();
        //     //
        //     let status = pkc.post_aqua_chain(thing).await.unwrap();

        //     println!("status: {:?}", status)
        // }

        // deprecated
        // Commands::PushRevision { file } => {
        //     let thing: pkc_api::da::ExportImportRevision =
        //         serde_json::from_reader(std::fs::File::open(file).unwrap()).unwrap();

        //     pkc.post_revision(thing).await.unwrap();

        //     println!(
        //         "status: {:?}",
        //         "probably okay, there was unit here and clippy was complaining."
        //     )
        // }
        Commands::List {} => {
            let hashes = pkc.list().await.expect("failed to list all revisions");

            println!("{:?}", hashes);
        }
        /*
        Commands::GetAllRevisions { page_title } => {
            let all_revisions = pkc.da_get_page_all_revs_full(&page_title).await.expect("failed to get all revisions");

            println!("{:?}", all_revisions);
        }
        */
        /*Commands::RecentChanges { time } => {
            let changes = pkc.mw_recent_changes(time).await.unwrap();

            dbg!(changes);
        }*/
        /*
        Commands::Update {} => {
            pkc.update_handler(|hash| {
                dbg!(hash);
            })
            .await
            .unwrap();
        }*/
        /*
        Commands::GetRecentChanges { timestamp, deleted } => {
            let changes = pkc
                .da_get_recent_changes(Timestamp::from_str(&timestamp).unwrap(), deleted)
                .await
                .unwrap();
                //this stoped working for some reason
            dbg!(changes);
        }
        Commands::PushRevisionBypass { file } => {
            let thing: pkc_api::da::ExportImportRevision =
                serde_json::from_reader(std::fs::File::open(file).unwrap()).unwrap();

            pkc.da_import_bypass(thing).await.unwrap();

            println!(
                "status: {:?}",
                "probably okay, there was unit here and clippy was complaining."
            )
        }*/
    }
}
