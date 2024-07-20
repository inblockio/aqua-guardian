use contract_interpreter::*;
use std::assert;

#[test]
fn test_list_rev() {
    // correct order of signatures
    const GOOD_FILES_LIST1: [&str; 3] = [
        include_str!(".././tests/test_data/DAA_SIG_SENDER_NO_WIT.json"),
        include_str!(".././tests/test_data/DAA_SIG_RECEIVER_NO_WIT.json"),
        include_str!(".././tests/test_data/DAA_SIG_RECEIVER_2_NO_WIT.json"),
    ];

    // incorrect order of signatures
    const BAD_FILES_LIST1: [&str; 3] = [
        include_str!(".././tests/test_data/DAA_SIG_RECEIVER_2_NO_WIT.json"),
        include_str!(".././tests/test_data/DAA_SIG_RECEIVER_NO_WIT.json"),
        include_str!(".././tests/test_data/DAA_SIG_SENDER_NO_WIT.json"),
    ];

    // const GOOD_SINGLE_REVISION: &str =
    //     include_str!(".././tests/test_data/DAA_SIG_SENDER_NO_WIT_NO_TERM.json");
    //const BAD_SINGLE_REVISION =

    //const GOOD_SINGLE_REVISION: &str = include_str!(".././tests/test_data/DAA_SIG_SENDER_NO_WIT_NO_TERM.json");
    //const BAD_SINGLE_REVISION =

    let mut rev_vec: Vec<&_> = Vec::new();

    /////////////////////////   TESTING OF GOOD LIST    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    let rev_one_v1_1 = &serde_json::from_str(GOOD_FILES_LIST1[0]).expect("cant parse json");
    let rev_one = verifier::v1_2::rev_v1_1_to_rev_v1_2(rev_one_v1_1, None, None);

    let rev_two_v1_1 = &serde_json::from_str(GOOD_FILES_LIST1[1]).expect("cant parse json");
    let rev_two = verifier::v1_2::rev_v1_1_to_rev_v1_2(rev_two_v1_1, Some(rev_one_v1_1), None);

    let rev_three_v1_1 =
        serde_json::from_str(GOOD_FILES_LIST1[2]).expect("failed to parse revision");

    let rev_three = verifier::v1_2::rev_v1_1_to_rev_v1_2(&rev_three_v1_1, Some(rev_two_v1_1), None);

    rev_vec.push(&rev_three);
    rev_vec.push(&rev_two);
    rev_vec.push(&rev_one);

    assert!(_test_list_revisions(rev_vec.iter().copied()).unwrap());
    rev_vec.clear();
    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    /////////////////////////   TESTING OF BAD LIST    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    let rev_one_v1_1 = &serde_json::from_str(BAD_FILES_LIST1[0]).expect("cant parse json");
    let rev_one = verifier::v1_2::rev_v1_1_to_rev_v1_2(rev_one_v1_1, None, None);

    let rev_two_v1_1 = &serde_json::from_str(BAD_FILES_LIST1[1]).expect("cant parse json");
    let rev_two = verifier::v1_2::rev_v1_1_to_rev_v1_2(rev_two_v1_1, Some(rev_one_v1_1), None);

    let rev_three_v1_1 =
        serde_json::from_str(BAD_FILES_LIST1[2]).expect("failed to parse revision");

    let rev_three = verifier::v1_2::rev_v1_1_to_rev_v1_2(&rev_three_v1_1, Some(rev_two_v1_1), None);

    rev_vec.push(&rev_three);
    rev_vec.push(&rev_two);
    rev_vec.push(&rev_one);

    assert!(!_test_list_revisions(rev_vec.iter().copied()).unwrap());
    rev_vec.clear();
    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    /////////////////////////   TESTING OF BAD LIST    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    let rev_one_v1_1 = &serde_json::from_str(BAD_FILES_LIST1[0]).expect("cant parse json");
    let rev_one = verifier::v1_2::rev_v1_1_to_rev_v1_2(rev_one_v1_1, None, None);

    let rev_two_v1_1 = &serde_json::from_str(BAD_FILES_LIST1[1]).expect("cant parse json");
    let rev_two = verifier::v1_2::rev_v1_1_to_rev_v1_2(rev_two_v1_1, Some(rev_one_v1_1), None);

    let rev_three_v1_1 =
        serde_json::from_str(BAD_FILES_LIST1[2]).expect("failed to parse revision");

    let rev_three = verifier::v1_2::rev_v1_1_to_rev_v1_2(&rev_three_v1_1, Some(rev_two_v1_1), None);

    rev_vec.push(&rev_three);
    rev_vec.push(&rev_two);
    rev_vec.push(&rev_one);

    assert!(!_test_list_revisions(rev_vec.iter().copied()).unwrap());
    rev_vec.clear();
    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
}

fn _test_list_revisions<'a>(
    revision: impl Iterator<Item = &'a verifier::v1_2::Revision>,
) -> Option<bool> {
    let mut rec_vec: Vec<_> = Vec::new();

    for rev in revision {
        let contract = Contract::from_revision(rev)?.ok()?;
        let state = Contract::sequence_number(&contract, rev);
        println!("contract : {:?} \n state: {:?}", contract, state);
        rec_vec.push((contract, state));
    }

    Some(is_contract_effective(
        rec_vec.iter().map(|(contract, state)| (contract, *state)),
    ).is_some())
}
