use guardian_common::custom_types::*;

use crate::storage::RevContext;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ExportBranch {
    namespace: i32,
    title: String,
    hashes: Vec<Hash>,
}

/// [`da::get_branch`](self)
impl super::Pkc {
    /// ## [`/rest.php/data_accounting/get_branch/<last_revision_hash>`][pkc_api_url]
    ///
    /// retrieves a list of the aqua-protocol [`Revision`] for a branch of specified revision_hashes (verification_hash).
    pub async fn da_get_branch(
        &self,
        last_revision_hash: Hash,
    ) -> super::Result<Branch<RevContext>> {
        let mut url = self.data_accounting_url("get_branch");
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push(&last_revision_hash.to_stackstr());
        }
        let resp = self.client.get(url).send().await?;

        let branch: ExportBranch = parse!(resp)?;

        let context = crate::storage::RevContext {
            namespace: branch.namespace,
            name: branch.title,
            genesis_hash: *branch.hashes.last().unwrap(),
            domain_id: "42".to_string(), // #TODO replace placeholder
        };

        Ok(Branch {
            metadata: context,
            hashes: branch.hashes,
        })
    }
}
