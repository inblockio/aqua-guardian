use super::*;

macro_rules! works {
    ($($test_case:ident),* $(,)?) => { $(
        #[test]
        fn $test_case() {
            const TEST_DATA: &str = include_str!(stringify!($test_case.json));
            let parsed: Revision = serde_json::from_str(TEST_DATA).unwrap();
            assert!(parsed.verify())
        }
    )* };
}

// todo: fix hashchain import
works!(testing_rev); //, testing_wit);

#[test]
fn verify_page() {
    const TEST_DATA: &str = include_str!("testing_rev.json");
    let parsed: Revision = serde_json::from_str(TEST_DATA).unwrap();
    let compare: Hash = "750671012c0afa8c5ea113391f48a28bfcb321baba40d0701c6b320eb2833e6eaf07998a39b6f4a2647cf061c0d17c2bb693ada9a1cd33b0b2b6896b846ff98f".parse().unwrap();
    assert_eq!(Revision::calculate_hash(&parsed), compare)
}
