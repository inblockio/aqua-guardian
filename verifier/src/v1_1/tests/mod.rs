use super::*;
use pkc_api::da::HashChain;

macro_rules! works {
    ($($test_case:ident),* $(,)?) => { $(
        #[test]
        #[ignore]
        fn $test_case() {
            const TEST_DATA: &str = include_str!(stringify!($test_case.json));
            let parsed: HashChain = serde_json::from_str(TEST_DATA).expect("parsing error");
            let (a, b) = hash_chain_integrity(&parsed);
            assert_eq!(
                a.bits(),
                0,
                "parsed: {parsed:?}\nrevisions: {a:?}\nhash chain integrity: {b:?}"
            );
        }
    )* };
}

// todo: fix hashchain import
works!(simple, signature, main_page);
