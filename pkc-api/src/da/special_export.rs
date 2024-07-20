use super::*;

// impl super::Pkc {
//     pub async fn get_page(
//         &self,
//         page_name: &str,
//     ) -> super::Result<export_import_entity::ExportImportEntity> {
//         let mut url = self.get_url("index.php");
//         url.query_pairs_mut()
//             .append_pair("title", "Special:VerifiedExport/export");
//         url.query_pairs_mut().append_pair("titles", page_name);
//
//         let resp = self.do_req(reqwest::Method::GET, url).await?;
//         // todo!("parse");
//
//         parse!(resp)
//     }
// }
impl super::Pkc {
    pub async fn da_special_export(&self, titles: &str) -> super::Result<UserFile> {
        let resp = self
            .client
            .get(self.get_url("index.php"))
            .query(&[
                ("title", "Special:VerifiedExport/export"),
                ("titles", titles),
            ])
            .send()
            .await?;

        parse!(resp)
    }

    pub async fn chain_from_genesis_hash(&self, genesis_hash: Hash) -> super::Result<UserFile> {
        let info = self
            .da_get_hash_chain_info_by_genesis_hash(genesis_hash)
            .await?;

        self.da_special_export(&info.title).await
    }
}

// impl super::Pkc {
//     pub async fn get_aqua_chain(
//         &self,
//         last_revision_hash: hash::Hash,
//     ) -> super::Result<export_import_entity::ExportImportEntity> {
//         todo!("implement the last_revision_hash -> aqua_chain API");
//         let mut url = self.get_url("index.php");
//         url.query_pairs_mut()
//             .append_pair("title", "Special:VerifiedExport/export");
//         url.query_pairs_mut().append_pair("titles", page_name);
//
//         let resp = self.do_req(reqwest::Method::GET, url).await?;
//         // todo!("parse");
//
//         parse!(resp)
//     }
// }
