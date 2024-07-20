#[derive(Clone, Copy, PartialEq, Eq)]
/// Represents a sp256k1 public key that has been used to sign an Aqua-Chain
pub struct Signature {
    pub recovery_id: libsecp256k1::RecoveryId,
    pub signature: libsecp256k1::Signature,
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_stackstr()[..])
    }
}

impl Signature {
    pub fn to_stackstr(self) -> super::StackStr<{ 2 + 2 * 65 }> {
        let mut s = [0u8; 2 + 2 * 65];
        s[0] = b'0';
        s[1] = b'x';
        let arr: [u8; 65] = self.into();
        // Safety: This will never error as it has exactly enough space in the buffer
        unsafe {
            hex::encode_to_slice(arr, &mut s[2..]).unwrap_unchecked();
        }
        super::StackStr(s)
    }
}

impl From<(libsecp256k1::Signature, libsecp256k1::RecoveryId)> for Signature {
    fn from(value: (libsecp256k1::Signature, libsecp256k1::RecoveryId)) -> Self {
        Signature { recovery_id: value.1, signature: value.0 }
    }
}

#[repr(C)]
struct EncSignature {
    signature: [u8; 64],
    recovery_id: u8,
}

impl From<Signature> for [u8; 65] {
    fn from(value: Signature) -> Self {
        let enc_sign = EncSignature {
            signature: value.signature.serialize(),
            recovery_id: value.recovery_id.serialize() + 27, //Magic number, consult ducks
        };
        unsafe { std::mem::transmute(enc_sign) }
    }
}
impl TryFrom<[u8; 65]> for Signature {
    type Error = libsecp256k1::Error;

    fn try_from(value: [u8; 65]) -> Result<Self, Self::Error> {
        let enc_sign: EncSignature = unsafe { std::mem::transmute(value) };
        Ok(Signature {
            signature: libsecp256k1::Signature::parse_standard(&enc_sign.signature)?,
            recovery_id: libsecp256k1::RecoveryId::parse_rpc(enc_sign.recovery_id)?,
        })
    }
}

// impl Default for Signature {
//     fn default() -> Self {
//         Self {
//             recovery_id: libsecp256k1::RecoveryId::parse(0).unwrap(),
//             signature: libsecp256k1::Signature::parse_overflowing(&[0u8; 64]),
//         }
//     }
// }
#[derive(thiserror::Error, Debug)]
pub enum ReadError {
    #[error("ascii or smth idk")]
    NotAsciiLower,
    #[error("goddamit WITH prefix")]
    NoPrefix,
    #[error("stay with HEX inputs only")]
    NotHex,
    #[error("libsecp256k1: {0}")]
    DecryptFail(#[from] libsecp256k1::Error)
}

impl std::str::FromStr for Signature {
    type Err = ReadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_ascii_lowercase() != s {
            return Err(ReadError::NotAsciiLower);
        }
        let s = s.strip_prefix("0x").ok_or(ReadError::NoPrefix)?;
        let h = super::from_hex(s).ok_or(ReadError::NotHex)?;
        h.try_into().map_err(ReadError::DecryptFail)
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        s.parse()
            .map_err(|problem| serde::de::Error::custom(format!("Signature problem is: {}", problem)))
    }
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = [0u8; 2 + 2 * 65];
        s[0] = b'0';
        s[1] = b'x';
        let arr: [u8; 65] = (*self).into();
        // Safety: This will never error as it has exactly enough space in the buffer
        unsafe {
            hex::encode_to_slice(arr, &mut s[2..]).unwrap_unchecked();
        }
        serializer.serialize_str(super::StackStr(s).as_ref())
    }
}

#[test]
fn test_read() {
    const TEST_DATA: &str = 
        "0xf0d0cadd0c82ade49db1e3443615dca67856e94b85d5590a2970d442e09b96e66fe9326f55a1e24b95f960f985bb524200be428d7084833db9ce7e778e2932121c";
    let _encoded_str: Signature =
        std::str::FromStr::from_str(TEST_DATA).expect("Correct Signature not read.");
    //dbg!(_encoded_str);
    const TEST_DATA_TOO_LONG: &str =
        "0xf0d0cadd0c82ade49db1e3443615dca67856e94b85d5590a2970d442e09b96e66fe9326f55a1e24b95f960f985bb524200be42048b18d7084833db9ce7e778e2932121c";
    <Signature as std::str::FromStr>::from_str(TEST_DATA_TOO_LONG)
        .expect_err("Accepted overly long signature");
    const TEST_DATA_NOPREFIX: &str =
        "f0d0cadd0c82ade49db1e3443615dca67856e94b85d5590a2970d442e09b96e66fe9326f55a1e24b95f960f985bb524200be428d7084833db9ce7e778e2932121c";
    <Signature as std::str::FromStr>::from_str(TEST_DATA_NOPREFIX)
        .expect_err("Accepted signature without 0x prefix.");
    const TEST_DATA_WITH_UPPER: &str = 
        "0xf0d0cadd0c82aDe49db1e3443615dca67856E94b85D5590a2970d442e09b96E66fe9326f55A1e24b95f960f985bb524200be428d7084833db9ce7e778e2932121C";
    <Signature as std::str::FromStr>::from_str(TEST_DATA_WITH_UPPER)
        .expect_err("Accepted signature with miXeD caSe.");
}

#[test]
fn test_write() {
    const TEST_DATA: &str = "0x52e60271ddeb607df95393b41d941f716de90ea7a901067b9f112aa5b737b8cc5c940b9374c950e518c06972a18feecff7b303977c0baf029b64e99b5754b4cf1c";
    let signature_thing: Signature = TEST_DATA.parse().expect("Correct Signature not read.");
    assert_eq!(TEST_DATA, &*signature_thing.to_stackstr(), "stuff broke");
}
