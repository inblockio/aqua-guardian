use contract_interpreter::*;
//use contract_interpreter::contract_handler::{detect_contract, generic_contract, ContractType};
use guardian_common::custom_types::*;

/// main() gets revision. Contract inside is to be or not to be detected
fn main() {
    // // For manual test of identity claim
    // const REV_TEST_IDENTITY_CLAIM: &str = r##"
    // {
    //     "verification_context": {
    //         "has_previous_signature": false,
    //         "has_previous_witness": false
    //     },
    //     "content": {
    //         "rev_id": 59,
    //         "content": {
    //             "main": "{{Identity Claim\n|account_guardian=0x04062274ed5bba92b9ab6b8687a86d87066d3dbac83e4f7e0e996a4d163e1bb294a75d8bbef8c9b2425bf7c020c7fe298580bc37fe8562227cb50e574dabb79701\n|account_auth_user=0x04062274ed5bba92b9ab6b8687a86d87066d3dbac83e4f7e0e996a4d163e1bb294a75d8bbef8c9b2425bf7c020c7fe298580bc37fe8562227cb50e574dabb79701\n}}",
    //             "transclusion-hashes": "[{\"dbkey\":\"Identity_Claim\",\"ns\":10,\"verification_hash\":\"e9bb93fc861e2825d2c5866c496e04f209dd3f25615a046f114b979c1869a138300e4890cf9bb246ab7ea6dd7eb3a57cc2b975ec8790fb4eb6392484f022bea6\"}]"
    //         },
    //         "content_hash": "173efeb75949b1316ed3d45bfd8eaf7fd92fad59d8a3d7b1840a25ec296b1140463327674fb82861bb613984adc536fbfb2ad26cd4a356b8ed99f62bf4fe2920"
    //     },
    //     "metadata": {
    //         "domain_id": "6908fdc00d",
    //         "time_stamp": "20240531173156",
    //         "previous_verification_hash": "",
    //         "metadata_hash": "241f29465b9cf355a1084e46ce15c452000a8bb0275689a3485665088af93da9e1e3d46c0ba70a17e5bfb5430fc1f3c80db739d76097597d77252f28e501408c",
    //         "verification_hash": "536189f7dd1a363e311094b89b9c3b085cd734b347ab0c08e816c8402861d4bf2ec7f887350f44264e0e67dee817c3e1d826ea34583299bd48969bf233954ebd"
    //     },
    //     "signature": null,
    //     "witness": null
    // }
    //     "##;
    // let revision: Revision =
    //          serde_json::from_str(REV_TEST_IDENTITY_CLAIM).expect("failed to parse Identity Claim");

    // For manual test of Data Access Agreement
    // const REV_TEST_DAA_SIG_SENDER_NO_WIT_NO_TERM: &str = r##"
    // {
    //     "verification_context": {
    //         "has_previous_signature": false,
    //         "has_previous_witness": false
    //     },
    //     "content": {
    //         "rev_id": 22,
    //         "content": {
    //             "main": "{{Data Access Agreement\n|sender=0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06\n|receiver=0xa2026582B94FEb9124231fbF7b052c39218954C2\n|terms=\n|pages=Main Page\n}}",
    //             "transclusion-hashes": "[{\"dbkey\":\"Data_Access_Agreement\",\"ns\":10,\"verification_hash\":\"7fcbef23b494aefc2f04a10d00d39d6873a14a429230e29e8f609cd94a691196fadcfc2f46160f20e346cdd96abb4ad62d4597e5ccc83ddabce096548b4e1daa\"},{\"dbkey\":\"Main_Page\",\"ns\":0,\"verification_hash\":\"dd36b9f09b756a4ec4be07519df36a827dd46a6f4ab75700a9f422f5ae06da28800424fcb5b45263efc1269e21c8d54bf6e9d49e7f43a4099bc0b4f3ae1c79a4\"}]"
    //         },
    //         "content_hash": "2c80e33db941f2033847c1bda52e52c4b6a8c84f61da96472c88437730ac48158f2b3e9e7b33364a1194b09f323b859bd245135a07ffa41f8007959ffb288727"
    //     },
    //     "metadata": {
    //         "domain_id": "7f31de4594",
    //         "time_stamp": "20240605133938",
    //         "previous_verification_hash": "",
    //         "metadata_hash": "0862860026d7f7f284861a2dab97aa9525777aa4c1b5f5cf94b1f12367d4969c02e6abebd75a64901a5a6fcd3568278b1818e95118c4523c129715755b3af404",
    //         "verification_hash": "ae0372e6bb85057e9e2c861dedccfb7b2d037d3332c081e0962bea229484d17ea770428d35f85e47ee8f3ace803896b197b5be8c290630fd1974781451f2ed30"
    //     },
    //     "signature": {
    //         "signature": "0xfd192cad0bf4cf6f30b89205db6cd621eec5d5ba35751aba1ed552bdc7d4fe16475d71d6f977677b99b347f216a4e3fd950eb5ab0ee02e1bdf38d361d3808eb11c",
    //         "public_key": "0x04c1a980bdf74ec29239adf7a675de2459cfe3b80a0d6514ba260ea5980f0ebc0e386054716ba9d4693a73afe1ad90b7165aabc7a0167b9cafcefd6d9bdef3dd2f",
    //         "wallet_address": "0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06",
    //         "signature_hash": "e6f717e589c49fd4f67135d602282f90f2da4f6a9c03d2239fb1fb054c878315e9ff5e6b6fac35bb7b495e0a578dc098fc1d5355e289047463d411d432bef434"
    //     },
    //     "witness": null
    // }"##;

    const REV_TEST_DAA_SIG_SENDER_NO_WIT: &str = r##"
    {
        "verification_context": {
            "has_previous_signature": true,
            "has_previous_witness": false
        },
        "content": {
            "rev_id": 23,
            "content": {
                "main": "{{Data Access Agreement\n|sender=0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06\n|receiver=0xa2026582B94FEb9124231fbF7b052c39218954C2\n|terms=JUST TAKE IT\n|pages=Main Page\n}}",
                "signature-slot": "[\n    {\n        \"user\": \"0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06\",\n        \"timestamp\": \"20240605135934\"\n    }\n]",
                "transclusion-hashes": "[{\"dbkey\":\"Data_Access_Agreement\",\"ns\":10,\"verification_hash\":\"7fcbef23b494aefc2f04a10d00d39d6873a14a429230e29e8f609cd94a691196fadcfc2f46160f20e346cdd96abb4ad62d4597e5ccc83ddabce096548b4e1daa\"},{\"dbkey\":\"Main_Page\",\"ns\":0,\"verification_hash\":\"dd36b9f09b756a4ec4be07519df36a827dd46a6f4ab75700a9f422f5ae06da28800424fcb5b45263efc1269e21c8d54bf6e9d49e7f43a4099bc0b4f3ae1c79a4\"}]"
            },
            "content_hash": "4ef34e0c9ddb77bfb936aa9eaf16ad2bcb335526904c26c5afec555a0cbecff4aa5d89e6346d4c70f59f43c97b8d342f8e07c39b7be21aa1a7e92cb9bb5edb7d"
        },
        "metadata": {
            "domain_id": "647403dbf7",
            "time_stamp": "20240605135934",
            "previous_verification_hash": "ae0372e6bb85057e9e2c861dedccfb7b2d037d3332c081e0962bea229484d17ea770428d35f85e47ee8f3ace803896b197b5be8c290630fd1974781451f2ed30",
            "metadata_hash": "45003e4d0ab45985d854c0744594ec2835b00a8fb16ff7bd589aefcd760d3f308b9dfd68be1d4b89691a03688cddfa9a658e1e376a081a97c1d6600fcaac588b",
            "verification_hash": "081449157f6ed1d66b5b2b5f7070f1c45ee9f9abbb9f11f5aef487018e0164a11995ed26283231098bf27b28cbf93279a498f9a1ce82ec142aec679f9d22c0f5"
        },
        "signature": {
            "signature": "0xfd192cad0bf4cf6f30b89205db6cd621eec5d5ba35751aba1ed552bdc7d4fe16475d71d6f977677b99b347f216a4e3fd950eb5ab0ee02e1bdf38d361d3808eb11c",
            "public_key": "0x04c1a980bdf74ec29239adf7a675de2459cfe3b80a0d6514ba260ea5980f0ebc0e386054716ba9d4693a73afe1ad90b7165aabc7a0167b9cafcefd6d9bdef3dd2f",
            "wallet_address": "0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06",
            "signature_hash": "e6f717e589c49fd4f67135d602282f90f2da4f6a9c03d2239fb1fb054c878315e9ff5e6b6fac35bb7b495e0a578dc098fc1d5355e289047463d411d432bef434"
        },
        "witness": null
    }
        "##;

    const REV_TEST_DAA_SIG_RECEIVER_NO_WIT: &str = r##"
    {
        "verification_context": {
            "has_previous_signature": true,
            "has_previous_witness": false
        },
        "content": {
            "rev_id": 25,
            "content": {
                "main": "{{Data Access Agreement\n|sender=0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06\n|receiver=0xa2026582B94FEb9124231fbF7b052c39218954C2\n|terms=JUST TAKE IT\n|pages=Main Page\n}}",
                "signature-slot": "[\n    {\n        \"user\": \"0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06\",\n        \"timestamp\": \"20240605135934\"\n    },\n    {\n        \"user\": \"0xa2026582B94FEb9124231fbF7b052c39218954C2\",\n        \"timestamp\": \"20240605141100\"\n    }\n]",
                "transclusion-hashes": "[{\"dbkey\":\"Data_Access_Agreement\",\"ns\":10,\"verification_hash\":\"7fcbef23b494aefc2f04a10d00d39d6873a14a429230e29e8f609cd94a691196fadcfc2f46160f20e346cdd96abb4ad62d4597e5ccc83ddabce096548b4e1daa\"},{\"dbkey\":\"Main_Page\",\"ns\":0,\"verification_hash\":\"dd36b9f09b756a4ec4be07519df36a827dd46a6f4ab75700a9f422f5ae06da28800424fcb5b45263efc1269e21c8d54bf6e9d49e7f43a4099bc0b4f3ae1c79a4\"}]"
                },
            "content_hash": "e119d5939c0262436524b95c75cf9b2c3a6fbb7f5d1fced393fea30659b35da1bf9af5b879b166a57485bcf3e7e75618814979ed57815fd11fc5e4283efa8725"
                    },
        "metadata": {
            "domain_id": "647403dbf7",
            "time_stamp": "20240605141100",
            "previous_verification_hash": "081449157f6ed1d66b5b2b5f7070f1c45ee9f9abbb9f11f5aef487018e0164a11995ed26283231098bf27b28cbf93279a498f9a1ce82ec142aec679f9d22c0f5",
            "metadata_hash": "184776c7c94437419ae193509dc35bc13839c6cd8d7ea3c2959c7cf73d88ab7347c9d067f4199f05c5f754aa04f777afbe57396b57a4b23fcd61209c19f1d164",
            "verification_hash": "941cf078abd3a937d135f5243f006f2d1279008e5b6bd958fa848f0b4d88b9723142b171c989cce45fc4f84de48b879cf3d835de9555ce7f1606306b35a47a11"
        },
        "signature": {
            "signature": "0x3994faf5cb3a0d00564002c0306041ab71270d22a5f5da625508eee0f2aa8d9543b74f60f33c1ac2e26b2ba3a4aaa3f4ab592136f560c5cbd29e3a442419c2ea1c",
            "public_key": "0x041518581af65749b3ddc69889df3e5d229bc8ad79279a07ddeb368ade5e1592368c5ff3b69143d7a1e7cf64f7d0774a6724e6eaf138d318d07ddc30f6081ca89a",
            "wallet_address": "0xa2026582B94FEb9124231fbF7b052c39218954C2",
            "signature_hash": "b49a65612dc55c97275b54c9b7ad344a143041e9160619d077db51ba5863afc5dc1e2075c6d97d049a3bc95a5fc36274f61023707bb564c0bdd3f2a8fd84f3d6"
        },
        "witness": null
    }
        "##;

    const REV_TEST_DAA_SIG_RECEIVER_2_NO_WIT: &str = r##"
        {
            "verification_context": {
                "has_previous_signature": true,
                "has_previous_witness": false
            },
            "content": {
                "rev_id": 25,
                "content": {
                    "main": "{{Data Access Agreement\n|sender=0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06\n|receiver=0xa2026582B94FEb9124231fbF7b052c39218954C2\n|terms=JUST TAKE IT\n|pages=Main Page\n}}",
                    "signature-slot": "[\n    {\n        \"user\": \"0x95b4b2e6d579eb9D8c32B34f8ca6ab11a3849c06\",\n        \"timestamp\": \"20240605135934\"\n    },\n    {\n        \"user\": \"0xa2026582B94FEb9124231fbF7b052c39218954C2\",\n        \"timestamp\": \"20240605141100\"\n    }\n]",
                    "transclusion-hashes": "[{\"dbkey\":\"Data_Access_Agreement\",\"ns\":10,\"verification_hash\":\"7fcbef23b494aefc2f04a10d00d39d6873a14a429230e29e8f609cd94a691196fadcfc2f46160f20e346cdd96abb4ad62d4597e5ccc83ddabce096548b4e1daa\"},{\"dbkey\":\"Main_Page\",\"ns\":0,\"verification_hash\":\"dd36b9f09b756a4ec4be07519df36a827dd46a6f4ab75700a9f422f5ae06da28800424fcb5b45263efc1269e21c8d54bf6e9d49e7f43a4099bc0b4f3ae1c79a4\"}]"
                    },
                "content_hash": "e119d5939c0262436524b95c75cf9b2c3a6fbb7f5d1fced393fea30659b35da1bf9af5b879b166a57485bcf3e7e75618814979ed57815fd11fc5e4283efa8725"
                        },
            "metadata": {
                "domain_id": "647403dbf7",
                "time_stamp": "20240605141100",
                "previous_verification_hash": "081449157f6ed1d66b5b2b5f7070f1c45ee9f9abbb9f11f5aef487018e0164a11995ed26283231098bf27b28cbf93279a498f9a1ce82ec142aec679f9d22c0f5",
                "metadata_hash": "184776c7c94437419ae193509dc35bc13839c6cd8d7ea3c2959c7cf73d88ab7347c9d067f4199f05c5f754aa04f777afbe57396b57a4b23fcd61209c19f1d164",
                "verification_hash": "941cf078abd3a937d135f5243f006f2d1279008e5b6bd958fa848f0b4d88b9723142b171c989cce45fc4f84de48b879cf3d835de9555ce7f1606306b35a47a11"
            },
            "signature": {
                "signature": "0x3994faf5cb3a0d00564002c0306041ab71270d22a5f5da625508eee0f2aa8d9543b74f60f33c1ac2e26b2ba3a4aaa3f4ab592136f560c5cbd29e3a442419c2ea1c",
                "public_key": "0x041518581af65749b3ddc69889df3e5d229bc8ad79279a07ddeb368ade5e1592368c5ff3b69143d7a1e7cf64f7d0774a6724e6eaf138d318d07ddc30f6081ca89a",
                "wallet_address": "0xa2026582B94FEb9124231fbF7b052c39218954C2",
                "signature_hash": "b49a65612dc55c97275b54c9b7ad344a143041e9160619d077db51ba5863afc5dc1e2075c6d97d049a3bc95a5fc36274f61023707bb564c0bdd3f2a8fd84f3d6"
            },
            "witness": null
        }
            "##;

    let revision_one_v1_1: Revision = serde_json::from_str(REV_TEST_DAA_SIG_SENDER_NO_WIT)
        .expect("failed to parse Data Access Agreement");

    let revision_one = verifier::v1_2::rev_v1_1_to_rev_v1_2(&revision_one_v1_1, None, None);

    let revision_two_v1_1: Revision =
        serde_json::from_str(REV_TEST_DAA_SIG_RECEIVER_NO_WIT).expect("failed to parse revision");

    let revision_two =
        verifier::v1_2::rev_v1_1_to_rev_v1_2(&revision_two_v1_1, Some(&revision_one_v1_1), None);

    let revision_three_v1_1: Revision =
        serde_json::from_str(REV_TEST_DAA_SIG_RECEIVER_2_NO_WIT).expect("failed to parse revision");

    let revision_three =
        verifier::v1_2::rev_v1_1_to_rev_v1_2(&revision_three_v1_1, Some(&revision_two_v1_1), None);

    // let revision_no_terms_v1_1: Revision =
    //     serde_json::from_str(REV_TEST_DAA_SIG_SENDER_NO_WIT_NO_TERM)
    //         .expect("failed to parse revision");

    // let revision_no_terms =
    //     verifier::v1_2::rev_v1_1_to_rev_v1_2(&revision_no_terms_v1_1, None, None);

    let mut rev_vec: Vec<&_> = Vec::new();
    rev_vec.push(&revision_one);
    rev_vec.push(&revision_two);
    rev_vec.push(&revision_three);
    //rev_vec.push(&revision_no_terms);

    match test_list_revisions(rev_vec.iter().copied()) {
        Some(effectiveness) => match effectiveness {
            Some(contract) => {
                match contract{
                    ContractEffect::AccessAgreement((_, effects)) => {
                        match effects{
                            AccessAgreementEffects::Granted => {
                                println!("CONTRACT EFFECTIVE!\n PAGE WAS SHARED DIRECTLY!")
                            },
                            AccessAgreementEffects::Offered => {
                                println!("CONTRACT EFFECTIVE!\n DAA WAS SHARED TO RECEIVER BUT NOT THE PAGE!")
                            },
                            AccessAgreementEffects::Accepted => {
                                println!("CONTRACT EFFECTIVE!\n DAA WAS SHARED BACK TO SENDER!")
                            },
                        }
                    },
                    ContractEffect::GuardianServitude((_, effects)) => {
                        match effects{
                            GuardianServitudeEffects::Suggested => {

                            },
                            GuardianServitudeEffects::Declared => {

                            },
                            GuardianServitudeEffects::Accepted => {
                                
                            },
                        }
                    },
                    ContractEffect::TlsIdentityClaim((_, effects)) => {
                        match effects{
                            TlsIdentityClaimEffects::IdentityClaimed => todo!(),
                        }
                    },
                }
            },
            None => {
                println!("NONE (1)!")
            },
        },
        None => {
            println!("NONE (2)!")
        }
    }

    // match test_one_rev(&revision_one){
    //     Some(bool) => {
    //         match bool{
    //             true => println!("TRUE"),
    //             false => println!("FALSE"),
    //         }
    //     },
    //     None => {
    //         println!("NONE!")
    //     },
    // }
}

// pub fn test_one_rev(revision: &Revision) -> Option<bool>{
//     let contract = Contract::from_revision(revision)?.ok()?;
//     println!("contract parsed from revision");
//     let state = Contract::identify_revision(&contract, revision);
//     println!("state detected: {:?}", state);
//     //Contract::is_contract_effective(&[(&contract,state)])
// }

pub fn test_list_revisions<'a>(
    revision: impl Iterator<Item = &'a verifier::v1_2::Revision>,
) -> Option<Option<ContractEffect>> {
    let mut rec_vec: Vec<_> = Vec::new();

    for rev in revision {
        let contract = Contract::from_revision(rev)?.ok()?;
        let state = Contract::sequence_number(&contract, rev);
        println!("contract : {:?} \n state: {:?}", contract, state);
        rec_vec.push((contract, state));
    }
    Some(is_contract_effective(
        rec_vec.iter().map(|(contract, state)| (contract, *state)),
    ))
}
