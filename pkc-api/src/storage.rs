#![deny(warnings)]

use super::*;
use crate::da::*;
use guardian_common::custom_types::*;
#[cfg(test)]
use guardian_common::storage::Storage;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct RevContext {
    pub namespace: i32,
    pub name: String,
    pub genesis_hash: Hash, //todo remove some day
    pub domain_id: String,
}

impl guardian_common::storage::Storage for Pkc {
    type Error = error::Error;
    type Context = RevContext;

    /// hardcoded because there is currently no way for us to get the context and it is irrelevant as long as all the names are different
    async fn get_context(&self, hash: Hash) -> Result<Self::Context> {
        let ctx = self.da_get_branch(hash).await?.metadata;

        Ok(ctx)
    }

    /// this function pushes a revision to the pkc with some specified context
    async fn store(
        &self,
        // hash: Hash,
        rev: Revision,
        context: Self::Context,
    ) -> Result<()> {
        let thing: ExportImportRevision = ExportImportRevision {
            context: HashChainInfo {
                genesis_hash: context.genesis_hash,
                domain_id: context.domain_id,
                latest_verification_hash: rev.metadata.verification_hash,
                site_info: SiteInfo {},
                title: context.name,
                chain_height: 0,
                namespace: context.namespace,
            },
            revision: rev,
        };

        self.post_revision(thing).await
    }

    /// read takes a hash and gives you the corresponding revision
    fn read(&self, hash: Hash) -> impl std::future::Future<Output = Result<Revision>> + Send {
        self.da_get_revision(hash)
    }

    /// context: all revisions hold the verification hash of the previous revision
    /// exception: the first revision has no previous revision
    /// input: latest revision hash
    /// output: a vector of all revision hashes starting from the latest_verification_hash
    async fn get_branch(&self, hash: Hash) -> Result<Branch<RevContext>> {
        // let mut rev = self.da_get_revision(hash).await?;
        // let mut branch: Vec<Hash> = vec![];
        // branch.push(rev.metadata.verification_hash);
        // while let Some(prev_hash) = rev.metadata.previous_verification_hash {
        //     rev = self.da_get_revision(prev_hash).await?;
        //     branch.push(rev.metadata.verification_hash);
        // }
        // Ok(branch)
        // #TODO: test
        let branch = self.da_get_branch(hash).await?;

        Ok(branch)
    }

    /// context: the pkc identifies its Pages by their unique page names
    /// the Guardian uses revision hashes because they are not subject to user changes
    /// idealy this would be one api call
    async fn list(&self) -> Result<Vec<Hash>> {
        let x = self
            .da_get_recent_changes(chrono::NaiveDateTime::UNIX_EPOCH.into(), false)
            .await?;

        Ok(x.into_keys().collect())
    }

    /// gets all of the recent changes from the pkc api and puts them into a given function
    /// this process is repeated every second
    async fn update_handler<F: Fn(Hash, String) + Send + Sync>(
        &self,
        f: F,
    ) -> Result<std::convert::Infallible> {
        let mut last = self.creation;

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let now = chrono::Utc::now().naive_utc();
            dbg!(now);
            let Ok(rc) = self.da_get_recent_changes(last.into(), true).await else {
                eprintln!("error while trying to get recent changes... retrying");
                continue;
            };
            dbg!(&rc);
            let fref = &f;
            rc.iter()
                .for_each(|(hash, kind)| fref(*hash, kind.to_string()));
            eprintln!("done with this update, waiting for next tick");
            last = now;
        }
    }
}

#[tokio::test]
async fn storage_test() {
    // needs a running container, uses the imported
    let data_store = Pkc::new("http://localhost:9352").unwrap();
    data_store.get_branch("725c2b99a955a690e50a1f22f356a64b02c144dd5adcbc09ac09f861fe2cc45a47185d7a9f5ecc60af86c0e60545aabe8c8c9c34feff92ea1da511ec0e2ef2ac".parse().unwrap()).await.unwrap();
    let revision = data_store.read("725c2b99a955a690e50a1f22f356a64b02c144dd5adcbc09ac09f861fe2cc45a47185d7a9f5ecc60af86c0e60545aabe8c8c9c34feff92ea1da511ec0e2ef2ac".parse().unwrap()).await.unwrap();
    let context = data_store.get_context("725c2b99a955a690e50a1f22f356a64b02c144dd5adcbc09ac09f861fe2cc45a47185d7a9f5ecc60af86c0e60545aabe8c8c9c34feff92ea1da511ec0e2ef2ac".parse().unwrap()).await.unwrap();
    data_store.list().await.unwrap();
    // Some(result);
    data_store.store(revision, context).await.unwrap();
    // Some(result);
    // needs MW API V1.2
    // let result = data_store.da_get_hash_chain_info_by_genesis_hash("725c2b99a955a690e50a1f22f356a64b02c144dd5adcbc09ac09f861fe2cc45a47185d7a9f5ecc60af86c0e60545aabe8c8c9c34feff92ea1da511ec0e2ef2ac".parse().unwrap()).await.unwrap();
    // Some(result);
    // Works but needs the DAA to be imported
    // let result = data_store.da_get_hash_chain_info_by_title("Template:DataAccessAgreement").await.unwrap();
    // Some(result);
    // let result = data_store.da_get_page_all_revs("Template:DataAccessAgreement").await.unwrap();
    // Some(result);
    // let result = data_store.da_get_page_last_rev("Template:DataAccessAgreement").await.unwrap();
    // Some(result);
}
