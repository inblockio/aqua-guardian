use super::v1_1::*;
use guardian_common::custom_types::*;
use std::error::Error;
use std::sync::mpsc;
use std::thread;

pub fn validate(content: String, rpc: String) {
    let representation_json: PageData = match serde_json::from_str(content.as_str()) {
        Err(why) => {
            eprintln!("couldn't parse: {}", why);
            return;
        }
        Ok(repr) => repr,
    };

    for p in representation_json.pages {
        // vector of channels to write read results of individual revision validations
        let mut rs: Vec<mpsc::Receiver<(String, bool)>> = vec![];
        thread::scope(|s| {
            let mut handles: Vec<thread::ScopedJoinHandle<()>> = vec![];
            for (key, r) in p.revisions.iter() {
                let (tx, rx) = mpsc::channel();
                rs.push(rx);
                let handle = s.spawn(move || {
                    let vresult = revision_validation(key.to_string(), r);
                    tx.send(vresult).unwrap()
                });
                handles.push(handle);
            }
            for h in handles {
                h.join().unwrap();
            }
        });

        for r in rs {
            let (k3y, rec) = r.recv().unwrap();
            println!("got: {} for: {}", rec, k3y);
        }

        let chain_linear = chain_validation(p);
        if chain_linear {
            println!("chain linear integrity valid");
        } else {
            eprintln!("chain linear integrity broken");
        }
    }
}

pub fn revision_validation(key: String, r: &Revision) -> (String, bool) {
    (key, false)
}

pub fn chain_validation(p: HashChain) -> bool {
    let mut integrity = true;

    let mut currentHash: Option<Hash> = None;
    let mut lastHash: Option<Hash> = None;

    for (key, r) in p.revisions {
        if key != r.metadata.verification_hash {
            integrity = false;
            eprintln!(
                "key didnt match verification hash in metadata: {} {}",
                key, r.metadata.verification_hash
            );
        }

        if let Some(value) = lastHash {
            println!("lastHash has value: {}", value);
            if r.metadata.previous_verification_hash != lastHash {
                integrity = false;
                eprintln!(
                    "last hash didnt match current last verification hash in metadata: {:#?} {:#?}",
                    lastHash, r.metadata.previous_verification_hash
                );
            }
        }

        lastHash = Some(r.metadata.verification_hash);
    }

    integrity
}
