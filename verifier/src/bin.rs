use clap::Parser;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use verifier::sa::validate;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub file: Option<Vec<String>>,
    #[clap(short, long)]
    pub rpc: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let cli = Cli::parse();

    let rpcpath = match cli.rpc {
        Some(rpcpath) => rpcpath,
        None => {
            println!("No rpc provided to standalone validator. Use --rpc url to define one");
            println!("Falling back to default open testnet rpc");
            "https://ethereum-sepolia-rpc.publicnode.com".to_string()
        }
    };

    println!("Rpc: {}", rpcpath);

    let files = match cli.file {
        Some(files) => files,
        None => {
            eprintln!("No filepaths provided to standalone validator. Use --file path1..");
            panic!("Exiting with no files to validate");
        }
    };

    for f in files {
        let path = Path::new(&f);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => {
                eprintln!("couldn't open {}: {}", display, why);
                panic!("Exiting with file can not be found");
            }
            Ok(file) => file,
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => (),
        }

        validate(s, rpcpath.clone());
    }

    Ok(())
}
