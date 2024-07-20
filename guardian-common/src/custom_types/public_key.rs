use std::ops::Deref;

#[derive(Clone, Copy, PartialEq, Eq)]
/// Used to represent a sp256k1 public key
pub struct PublicKey(libsecp256k1::PublicKey);

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_stackstr()[..])
    }
}

impl PublicKey {
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

impl From<libsecp256k1::PublicKey> for PublicKey {
    fn from(value: libsecp256k1::PublicKey) -> Self {
        PublicKey(value)
    }
}

impl From<PublicKey> for [u8; 65] {
    fn from(value: PublicKey) -> Self {
        value.0.serialize()
    }
}
impl From<PublicKey> for ethaddr::Address {
    fn from(value: PublicKey) -> Self {
        use crate::prelude::*;
        let mut hasher = crypt::Keccak256::default();
        hasher.update(&<[u8; 65]>::from(value)[1..]);
        let bytes32: [u8; 32] = hasher.finalize().into();

        ethaddr::Address(bytes32[12..].try_into().unwrap())
    }
}

impl TryFrom<[u8; 65]> for PublicKey {
    type Error = libsecp256k1::Error;

    fn try_from(value: [u8; 65]) -> Result<Self, Self::Error> {
        libsecp256k1::PublicKey::parse(&value).map(Self)
    }
}

impl Deref for PublicKey {
    type Target = libsecp256k1::PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::str::FromStr for PublicKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_ascii_lowercase() != s {
            return Err(());
        }
        let s = s.strip_prefix("0x").ok_or(())?;
        let h = super::from_hex(s).ok_or(())?;
        h.try_into().map_err(|_| ())
    }
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_stackstr().fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        s.parse()
            .map_err(|_| serde::de::Error::custom("not a valid signature (or maybe not supported)"))
    }
}

impl serde::Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_stackstr().as_ref())
    }
}

#[test]
fn test_read() {
    const TEST_DATA: &str =          "0x04062274ed5bba92b9ab6b8687a86d87066d3dbac83e4f7e0e996a4d163e1bb294a75d8bbef8c9b2425bf7c020c7fe298580bc37fe8562227cb50e574dabb79701";
    let _encoded_str: PublicKey =
        std::str::FromStr::from_str(TEST_DATA).expect("Correct public key not read.");
    //dbg!(_encoded_str);
    const TEST_DATA_TOO_LONG: &str = "0x04062274ed5bba92b9ab6b8687a86d87066d3dbac83e4f7e0e996a0484d163e1bb294a75d8bbef8c9b2425bf7c020c7fe298580bc37fe8562227cb50e574dabb79701";
    <PublicKey as std::str::FromStr>::from_str(TEST_DATA_TOO_LONG)
        .expect_err("accepted overly long public key");
    const TEST_DATA_NOPREFIX: &str = "04062274ed5bba92b9ab6b8687a86d87066d3dbac83e4f7e0e996a4d163e1bb294a75d8bbef8c9b2425bf7c020c7fe298580bc37fe8562227cb50e574dabb79701";
    <PublicKey as std::str::FromStr>::from_str(TEST_DATA_NOPREFIX)
        .expect_err("accepted public key without 0x prefix.");
    const TEST_DATA_WITH_UPPER: &str = "0x04062274ed5bba92b9Ab6b8687a86d87066d3dbac83e4f7e0e996a4d163e1bB294a75d8bBef8c9b2425bf7c020c7Fe298580bc37fe8562227cb50e574dabb79701";
    <PublicKey as std::str::FromStr>::from_str(TEST_DATA_WITH_UPPER)
        .expect_err("accepted public key with Uppercase Letters.");
}

#[test]
fn test_write() {
    const TEST_DATA: &str = "0x04062274ed5bba92b9ab6b8687a86d87066d3dbac83e4f7e0e996a4d163e1bb294a75d8bbef8c9b2425bf7c020c7fe298580bc37fe8562227cb50e574dabb79701";
    let pubkey_thing: PublicKey = TEST_DATA.parse().expect("Correct public key not read.");
    assert_eq!(TEST_DATA, &*pubkey_thing.to_stackstr(), "stuff broke");
}
