use ethaddr::Address;
use guardian_common::prelude::*;
use serde::ser::SerializeStruct;

struct Message {
    address: Address,
    uri: reqwest::Url,
    nonce: String,
    issued_at: chrono::NaiveDateTime,
    expiration_time: chrono::NaiveDateTime,
    chain_id: i64,
    resources: Vec<String>,
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{} wants you to sign in with your Ethereum account:\n{}\n\nYou are signing-in to {}.\n\nURI: {}\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}\nExpiration Time: {}\nResources:",
            self.uri.domain().unwrap_or_default(),
            self.uri.port().map(|p| format!(":{}", p)).unwrap_or_default(),
            self.address,
            self.uri.host_str().unwrap_or_default(),
            &self.uri.as_str()[..self.uri.as_str().len()-1],
            self.chain_id,
            self.nonce,
            self.issued_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            self.expiration_time.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        ))?;
        for item in self.resources.iter() {
            f.write_fmt(format_args!("\n- {item}"))?;
        }
        Ok(())
    }
}
impl From<&Message> for libsecp256k1::Message {
    fn from(value: &Message) -> Self {
        let s = value.to_string();
        let mut msg = crypt::Keccak256::default();
        msg.update("\x19Ethereum Signed Message:\n");
        msg.update(format!("{}", s.as_bytes().len()));
        msg.update(s.as_bytes());
        libsecp256k1::Message::parse(&msg.finalize().into())
    }
}
impl serde::Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut obj = serializer.serialize_struct("message", 10)?;
        obj.serialize_field(
            "domain",
            &format!(
                "{}{}",
                self.uri.domain().unwrap_or_default(),
                self.uri
                    .port()
                    .map(|p| format!(":{}", p))
                    .unwrap_or_default()
            ),
        )?;
        obj.serialize_field("address", &self.address)?;
        obj.serialize_field(
            "statement",
            &format_args!(
                "You are signing-in to {}.",
                self.uri.host_str().unwrap_or_default()
            ),
        )?;
        obj.serialize_field("uri", &self.uri.as_str()[..self.uri.as_str().len() - 1])?;
        obj.serialize_field("version", "1")?;
        obj.serialize_field("nonce", &self.nonce)?;
        obj.serialize_field(
            "issuedAt",
            &self
                .issued_at
                .and_utc()
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        )?;
        obj.serialize_field(
            "expirationTime",
            &self
                .expiration_time
                .and_utc()
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        )?;
        obj.serialize_field("chainId", &self.chain_id)?;
        obj.serialize_field("resources", &self.resources)?;
        obj.end()
    }
}

#[derive(serde::Serialize)]
struct SiweCookie {
    message: Message,
    raw: String,
    signature: Signature,
}
/// url example: http://localhost:9352/
pub async fn login<S: guardian_common::signing::Signer>(
    signer: S,
    pkc_url: &reqwest::Url,
) -> reqwest::Client {
    let cookie_store = reqwest_cookie_store::CookieStore::new(None);
    let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
    let cookie_store = std::sync::Arc::new(cookie_store);
    // let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_store.clone())
        //Adding timeout for SIWE-OIDC API requests
        .timeout(std::time::Duration::from_secs(9))
        .build()
        .unwrap();
    // + index.php
    let loginpage_url = {
        let mut loginpage_url = pkc_url.clone();
        if let Ok(mut path_mut) = loginpage_url.path_segments_mut() {
            path_mut.push("index.php");
        }
        loginpage_url
    };
    let authorize_resp = client
        .get(loginpage_url)
        .query(&[
            ("title", "Special:UserLogin"),
            ("returnto", "Special:PluggableAuthLogin"),
        ])
        .send()
        .await
        .expect("failed to get mediawiki session cookie");
    // dbg!(&authorize_resp);
    let siwe_authorize_url = authorize_resp.url().clone();

    let _ = authorize_resp.error_for_status().expect("didn't work");

    let siwe_url = {
        let mut siwe_url = siwe_authorize_url.clone();
        siwe_url.set_path("/");
        siwe_url.set_query(None);
        siwe_url.set_fragment(None);
        siwe_url
    };
    let Some(nonce) = siwe_authorize_url
        .query_pairs()
        .find_map(|q| (&q.0 == "nonce").then_some(q.1.to_string()))
    else {
        panic!("siwe_authorize_url: {siwe_authorize_url} does not contain nonce");
    };
    let now = chrono::offset::Utc::now().naive_utc();
    let message = Message {
        address: signer.identity().into(),
        uri: siwe_url.clone(),
        nonce,
        issued_at: now,
        expiration_time: now + chrono::Duration::days(3),
        chain_id: 1,
        resources: vec![format!("{pkc_url}index.php/Special:PluggableAuthLogin")],
    };
    let signature = signer.sign(&(&message).into());
    let cookie = SiweCookie {
        raw: message.to_string(),
        message,
        signature,
    };
    eprintln!("{}", serde_json::to_string(&cookie).unwrap());
    {
        let not_url_encoded = serde_json::to_string(&cookie).expect("this shouldn't error");
        let keks =
            reqwest_cookie_store::RawCookie::new("siwe", uri_encode::encode_uri(not_url_encoded));
        let mut lock = cookie_store.lock().expect("failed to lock mutex");
        lock.insert_raw(&keks, &siwe_url)
            .expect("failed to insert siwe cookie into cookie store");
    }

    let siwe_sign_in_url = {
        let mut siwe_sign_in_url = siwe_authorize_url.clone();
        siwe_sign_in_url.set_path("sign_in");
        let it: Vec<_> = siwe_authorize_url
            .query_pairs()
            .filter(|a| ["redirect_uri", "state", "client_id", "oidc_nonce"].contains(&&*a.0))
            .collect();
        let mut query = siwe_sign_in_url.query_pairs_mut();
        query.clear();
        query.extend_pairs(it.into_iter());
        drop(query);
        siwe_sign_in_url
    };
    let sign_in_resp = client
        .get(siwe_sign_in_url)
        .send()
        .await
        .expect("failed sign in");

    let _ = sign_in_resp.error_for_status().expect("didn't work");
    // dbg!(&sign_in_resp);

    // let cookies: Vec<_> = sign_in_resp
    //     .cookies()
    //     .map(|cookie| {
    //         cookie_store::Cookie::try_from_raw_cookie(
    //             &reqwest_cookie_store::RawCookie::new(
    //                 cookie.name().to_string(),
    //                 cookie.value().to_string(),
    //             ),
    //             pkc_url,
    //         )
    //     })
    //     .collect();
    // let new_cookie_store = reqwest_cookie_store::CookieStore::from_cookies(cookies, false)
    //     .expect("did not correctly save cookies");

    // let mut f = std::fs::File::create("just_tell_me_your_cookies").unwrap();
    // new_cookie_store.save_json(&mut f).unwrap();

    // reqwest::Client::builder()
    //     .cookie_provider(std::sync::Arc::new(
    //         reqwest_cookie_store::CookieStoreRwLock::new(new_cookie_store),
    //     ))
    //     .build()
    //     .expect("failed")
    client
}
