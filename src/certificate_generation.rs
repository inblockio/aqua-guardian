// functions for cert generation. may be moved to seperate file later
pub fn gen_identity() {
    dotenv::dotenv().ok();
    let private_key: guardian_common::signing::SimpleSigner = std::env::var("PRIVATE_KEY")
        .expect("no private key")
        .parse()
        .expect("failed to parse private key");
    // let pkc_url: url::Url = std::env::var("PKC_URL")
    //     .expect("no pkc url")
    //     .parse()
    //     .expect("not a valid url");
    // let admin_user: ethaddr::Address = std::env::var("ADMIN_USER")
    //     .expect("no admin user")
    //     .parse()
    //     .expect("failed to parse ADMIN_USER");
    let host: std::net::IpAddr = std::env::var("HOST")
        .expect("no host")
        .parse()
        .expect("failed to parse host");
    let port: u16 = std::env::var("PORT")
        .expect("no port")
        .parse()    
        .expect("failed to parse PORT");

    if !std::path::Path::new("identity.pem").exists() {
        std::fs::write(std::path::Path::new("identity.pem"), "")
            .expect("failed to create identity.pem file");
    }

    if let Some(key_pair) = read_keys("identity") {
        let key_pair =
            rcgen::KeyPair::try_from(&key_pair).expect("failed to make tls into rcgen keypair");
        let key_pair_pem = key_pair.serialize_pem();

        let (cert, _) =
            crate::contract_generation::make_guardian_cert(key_pair, host, port, private_key)
                .unwrap();

        std::fs::write("identity.pem", format!("{}{}", cert.pem(), key_pair_pem))
            .expect("failed to store new cert")
    } else {
        // generate key and run again. should not lead to infinite loop unless write fails in a weird way.
        let key_pair = rcgen::KeyPair::generate().unwrap();
        std::fs::write("identity.pem", key_pair.serialize_pem()).unwrap();
        gen_identity();
    }
}

pub fn read_keys(name: &str) -> Option<webpki::types::PrivateKeyDer<'static>> {
    if !std::path::Path::new(&format!("{name}.pem")).exists() {
        panic!("no private key found try generate one using key-gen")
    }
    println!("reading keys"); // #TODO: remove
                              // read keypair from file
    let file = std::fs::File::open(format!("{name}.pem")).unwrap();
    let mut bufreader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    let client_keypair_der = rustls_pemfile::pkcs8_private_keys(&mut bufreader).next()?.ok()?.into();

    Some(client_keypair_der)
}

pub fn read_certs(
    name: &str,
) -> Result<Vec<webpki::types::CertificateDer<'static>>, Box<dyn std::error::Error + Send + Sync>> {
    println!("reading certs"); // #TODO: remove
                               // read certs
    let file = std::fs::File::open(format!("{name}.pem")).unwrap();
    let mut bufreader = std::io::BufReader::new(file);
    let client_cert_der = rustls_pemfile::certs(&mut bufreader).collect::<Result<_, _>>().unwrap();
    Ok(client_cert_der)
}