use siwe_oidc_auth::*;

#[tokio::test]
async fn login_test() {
    dotenv::dotenv().ok();
    let private_key: guardian_common::signing::SimpleSigner = std::env::var("PRIVATE_KEY")
        .expect("no private key")
        .parse()
        .expect("failed to parse private key");

    let pkc_url: reqwest::Url = std::env::var("PKC_URL")
        .expect("no pkc url")
        .parse()
        .expect("not a valid url");

    let _res = login(private_key, &pkc_url).await;
}
