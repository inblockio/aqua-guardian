// no sensible automated testing
fn main() {
    let private = libsecp256k1::SecretKey::random(&mut rand::thread_rng());
    let private_str = format!("0x{}", hex::encode(private.serialize()));
    std::env::set_var("PRIVATE_KEY", &private_str);
    let public = libsecp256k1::PublicKey::from_secret_key(&private);
    let public = guardian_common::custom_types::PublicKey::from(public);
    let public_str = public.to_stackstr();
    std::env::set_var("PUBLIC_KEY", &*public_str);
    let addr_str = ethaddr::Address::from(public).to_string();
    std::env::set_var("ADDRESS", &addr_str);
    std::fs::write(
        ".env",
        format!("PRIVATE_KEY={private_str}\nPUBLIC_KEY={public_str}\nADDRESS={addr_str}\n"),
    )
    .unwrap()
}
