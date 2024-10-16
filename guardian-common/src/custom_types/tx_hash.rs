#[derive(Hash, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
/// A transaction hash
pub struct TxHash([u8; 32]);

impl TxHash {
    pub fn to_stackstr(self) -> super::StackStr<66> {
        let mut data = [0u8; 2 + 32 * 2];
        data[0] = b'0';
        data[1] = b'x';
        // Safety: data is exactly the right size for the hex output
        unsafe {
            hex::encode_to_slice(self.0, &mut data[2..]).unwrap_unchecked();
        }
        super::StackStr(data)
    }
}

impl std::str::FromStr for TxHash {
    // todo: err
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = if s.starts_with("0x") {
            // If the string starts with "0x", just use it as is
            s
        } else {
            // If the string doesn't start with "0x", add it manually
            let mut prefixed = String::with_capacity(2 + s.len());
            prefixed.push_str("0x");
            prefixed.push_str(s);
            &prefixed.clone()
        };

        // Strip the "0x" now, and handle the rest as a hex string
        let s = s
            .strip_prefix("0x")
            .ok_or("HASH HAS NO '0x' PREFIX".to_string())?;

        // Ensure the hex string is the correct length (64 characters for 32 bytes)
        if s.len() != 64 {
            return Err("LENGTH NOT EQUAL TO 64".to_string());
        }

        // Decode the hex string into bytes
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(s, &mut bytes).map_err(|_| "UNABLE TO DECODE".to_string())?;
        Ok(TxHash(bytes))
    }
}

impl std::fmt::Display for TxHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_stackstr())
    }
}

impl std::ops::Deref for TxHash {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[u8; 32]> for TxHash {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}
impl From<TxHash> for [u8; 32] {
    fn from(val: TxHash) -> Self {
        val.0
    }
}

impl<'de> serde::Deserialize<'de> for TxHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        s.parse()
            .map_err(|_| serde::de::Error::custom("Invalid sha3_512 hash"))
    }
}

impl serde::Serialize for TxHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(&self.0[..]))
    }
}

#[test]
fn test_read() {
    const TEST_DATA: &str = "0x17cb36e3abfe5cd2894f7b324102c3864d202bc7b85e4f3e5ec78ca2c3db79d7";
    let _hash: TxHash = TEST_DATA.parse().expect("Rejected correct TxHash.");
    //dbg!(_hash);
    const TEST_DATA_NOPREFIX: &str =
        "17cb36e3abfe5cd2894f7b324102c3864d202bc7b85e4f3e5ec78ca2c3db79d7";
    <TxHash as std::str::FromStr>::from_str(TEST_DATA_NOPREFIX)
        .expect_err("Accepted TxHash without prefix.");
    const TEST_DATA_WITH_UPPER: &str =
        "0x17cb36e3abfe5cd2894f7b324102C3864d202Bc7b85e4f3e5ec78ca2c3db79d7";
    <TxHash as std::str::FromStr>::from_str(TEST_DATA_WITH_UPPER)
        .expect_err("Accepted TxHash wiTh miXed caSe.");
}

#[test]
fn test_write() {
    const TEST_DATA: &str = "0x17cb36e3abfe5cd2894f7b324102c3864d202bc7b85e4f3e5ec78ca2c3db79d7";
    let hash_thing: TxHash = TEST_DATA.parse().expect("Rejected correct TxHash.");
    //dbg!(_hash);
    assert_eq!(TEST_DATA, &*hash_thing.to_stackstr(), "Tx Hash failed.");
}
