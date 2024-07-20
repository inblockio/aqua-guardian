//! # JSON API types
//!
//! main objects appear here, subobjects appear in sub`mod`ules

import! {
    content::{RevisionContent, FileContent};
    metadata::{RevisionMetadata, ExportRevisionMetadata};
    signature::RevisionSignature;
    witness::RevisionWitness;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
/// A revision of a document on an Aqua chain
pub struct Revision {
    pub content: content::RevisionContent,
    pub metadata: metadata::RevisionMetadata,
    pub signature: Option<signature::RevisionSignature>,
    pub witness: Option<witness::RevisionWitness>,
}

#[test]
fn parse_revision_future() {
    const REV_TEST_PAGE_NO_SIG_NO_WIT: &str = r##"
{
    "verification_context": {
        "has_previous_signature": false,
        "has_previous_witness": false
    },
    "content": {
        "rev_id": 9,
        "content": {
            "main": "Welcome to the Personal Knowledge Container!<br>\n<i>Your place to own and govern your trusted data.<\/i><br>\n<i>Build on and with free and open source software.<\/i><br>\n\n''Follow our [[Interactive_Tutorial]] to learn how to use this product.''<br>\nYou can read about all of the custom Aqua actions in the [[PKC_Documentation]].\n\nThe [[Personal Knowledge Container]] is your private Data Vault.<br>\nData is structured according the [[Aqua Protocol]] to created trusted an [[Verified Data]]. <br> \n\nConfigure Mediawiki 'Data Accounting Extension' and find more information here: <\/i>[[Special:DataAccountingConfig|Configuration \u2699\ufe0f]]<i>",
            "transclusion-hashes": "[{\"dbkey\":\"Interactive_Tutorial\",\"ns\":0,\"verification_hash\":\"d95645ee3b14098949c18c6adbf1463752ded88cf8159a487be5cfcd92bddbf395692e8b01cc303d54c3a6ecb6c96fcb2b9e2e02308aa74ffdb7d056da98a944\"},{\"dbkey\":\"PKC_Documentation\",\"ns\":0,\"verification_hash\":null},{\"dbkey\":\"Personal_Knowledge_Container\",\"ns\":0,\"verification_hash\":null},{\"dbkey\":\"Aqua_Protocol\",\"ns\":0,\"verification_hash\":null},{\"dbkey\":\"Verified_Data\",\"ns\":0,\"verification_hash\":null}]"
        },
        "content_hash": "e98a114381ae546997987bb66c995fadef23c58a6fa9108eb52e6811e7947c4cbce2724089c62a924355793c7a7262b5cd02300d14ac4095d557bd01e9c9cd2c"
    },
    "metadata": {
        "domain_id": "3042fae0b0",
        "time_stamp": "20240522125324",
        "previous_verification_hash": null,
        "metadata_hash": "6be940f1fb4e9d503318fe41d5fa5defacf84ea28bb039de9b5e4e99d749639a135d79c021e85619153deabac45c7dc0a5b7deb9e3923da40a53f3954f4aaeba",
        "verification_hash": "d9e09f8529fed3b909876f34f21c7148d73de01d82f8aee43c52d9ee2601999ddcbf4593a19baac497d9d83bb98c94c2508b8157efafcd6484cbca7c4953af5f"
    },
    "signature": null,
    "witness": null
}
    "##;
    let _rev: Revision =
        serde_json::from_str(REV_TEST_PAGE_NO_SIG_NO_WIT).expect("failed to parse no sig no wit");
    //dbg!(_rev);
    const REV_TEST_PAGE_SIG_WIT: &str = r##"
{
    "verification_context": {
        "has_previous_signature": false,
        "has_previous_witness": false
    },
    "content": {
        "rev_id": 9,
        "content": {
            "main": "Welcome to the Personal Knowledge Container!<br>\n<i>Your place to own and govern your trusted data.<\/i><br>\n<i>Build on and with free and open source software.<\/i><br>\n\n''Follow our [[Interactive_Tutorial]] to learn how to use this product.''<br>\nYou can read about all of the custom Aqua actions in the [[PKC_Documentation]].\n\nThe [[Personal Knowledge Container]] is your private Data Vault.<br>\nData is structured according the [[Aqua Protocol]] to created trusted an [[Verified Data]]. <br> \n\nConfigure Mediawiki 'Data Accounting Extension' and find more information here: <\/i>[[Special:DataAccountingConfig|Configuration \u2699\ufe0f]]<i>",
            "transclusion-hashes": "[{\"dbkey\":\"Interactive_Tutorial\",\"ns\":0,\"verification_hash\":\"d95645ee3b14098949c18c6adbf1463752ded88cf8159a487be5cfcd92bddbf395692e8b01cc303d54c3a6ecb6c96fcb2b9e2e02308aa74ffdb7d056da98a944\"},{\"dbkey\":\"PKC_Documentation\",\"ns\":0,\"verification_hash\":null},{\"dbkey\":\"Personal_Knowledge_Container\",\"ns\":0,\"verification_hash\":null},{\"dbkey\":\"Aqua_Protocol\",\"ns\":0,\"verification_hash\":null},{\"dbkey\":\"Verified_Data\",\"ns\":0,\"verification_hash\":null}]"
        },
        "content_hash": "e98a114381ae546997987bb66c995fadef23c58a6fa9108eb52e6811e7947c4cbce2724089c62a924355793c7a7262b5cd02300d14ac4095d557bd01e9c9cd2c"
    },
    "metadata": {
        "domain_id": "3042fae0b0",
        "time_stamp": "20240522125324",
        "previous_verification_hash": null,
        "metadata_hash": "6be940f1fb4e9d503318fe41d5fa5defacf84ea28bb039de9b5e4e99d749639a135d79c021e85619153deabac45c7dc0a5b7deb9e3923da40a53f3954f4aaeba",
        "verification_hash": "d9e09f8529fed3b909876f34f21c7148d73de01d82f8aee43c52d9ee2601999ddcbf4593a19baac497d9d83bb98c94c2508b8157efafcd6484cbca7c4953af5f"
    },
    "signature": {
        "signature": "0xee00007e8eb51b2566240897ea4c9b1aee30bfc48929c3a3046855423fd43dba2fcd7a51e225eef0cc2dd561c147d733934f32c6ae11f9be490987e6b7fe93781c",
        "public_key": "0x04f00d6e178562a62ec9e595da4294f640dca429fc98e7128b8e7ee83039912d64a924bea34e629b9b45990c65e92efc3d74533f870479d10ff895834fff4fa1e8",
        "wallet_address": "0x1ad5da43de60aa7d311f9b4e9c3342c155e6d2e0",
        "signature_hash": "91bbc0bec6cc84cc11cf1747d514b96c34666291242b048c034ccf28d909b524d0c0fdbb5614a8e20b25ce855097a633292367769c5d18f04918e0660a77d494"
    },
    "witness": {
        "witness_event_id": 1,
        "domain_id": null,
        "domain_snapshot_title": "N\/A",
        "witness_hash": "593872fb126334e4e325055a81f5e7001a74e801f59ba992312e970eb00e16ef60ca0be581500ba8e0879f20a86f4040c6c973a57b2f476041ef3ce13a511d29",
        "domain_snapshot_genesis_hash": "305ca37488e0d1e20535f08f073290c564040f6574a84ab73fd5d4c6def175bc02260585bae9f6fc4a584a8367881ef5257c364692ff07378b6caa28d1450d9e",
        "merkle_root": "c2c84eb0f69b769493e39b6e86268957be98fe735b5782cfcbb49a216ec17684dabda30082212080bb522dc3665fb226ad4932f7d8e1baf5808efd08f38a2ac8",
        "witness_event_verification_hash": "39cff24a0eebc962ec1e5e78e69dc2ac508799c646f722a580d8ab58bcc523db225e64a10edcb43b2c511e6734793f179ee027c0207e1c328b014b820f146291",
        "witness_network": "goerli",
        "smart_contract_address": "0x45f59310ADD88E6d23ca58A0Fa7A55BEE6d2a611",
        "witness_event_transaction_hash": "0x17cb36e3abfe5cd2894f7b324102c3864d202bc7b85e4f3e5ec78ca2c3db79d7",
        "sender_account_address": "0x1ad5da43de60aa7d311f9b4e9c3342c155e6d2e0",
        "source": "imported",
        "structured_merkle_proof": [
            {
                "right_leaf": "6be940f1fb4e9d503318fe41d5fa5defacf84ea28bb039de9b5e4e99d749639a135d79c021e85619153deabac45c7dc0a5b7deb9e3923da40a53f3954f4aaeba",
                "left_leaf": "6be940f1fb4e9d503318fe41d5fa5defacf84ea28bb039de9b5e4e99d749639a135d79c021e85619153deabac45c7dc0a5b7deb9e3923da40a53f3954f4aaeba"
            }
        ]
    }
}
    "##;
    let _rev: Revision =
        serde_json::from_str(REV_TEST_PAGE_SIG_WIT).expect("failed to parse with sig and wit");
    //dbg!(_rev);
}

#[test]
fn main_page_signed() {
    const TEST_DATA: &str = include_str!("../../../contract-interpreter/tests/test_data/DAA_SIG_SENDER_NO_WIT.json");
    dbg!(TEST_DATA);
    let x: Revision = serde_json::from_str(TEST_DATA).expect("failed to parse");    
    dbg!(&x);
    assert!(x.signature.is_some());
}
