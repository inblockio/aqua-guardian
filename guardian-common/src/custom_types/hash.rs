#[derive(Hash, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
/// Used to represent Hashes
pub struct Hash(crate::crypt::Hash);

impl core::fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_stackstr()[..])
        // f.write_fmt(format_args!("{}..", &self.to_stackstr()[..20]))
    }
}

impl Hash {
    pub fn to_stackstr(self) -> super::StackStr<128> {
        let mut arr = [0; 128];
        // Safety: data is exactly the right size for the hex output
        unsafe {
            hex::encode_to_slice(self.0, &mut arr[..]).unwrap_unchecked();
        }
        super::StackStr(arr)
    }
}

impl std::str::FromStr for Hash {
    // todo: err
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Hash(super::from_hex(s).ok_or(())?.into()))
    }
}

impl From<[u8; 64]> for Hash {
    fn from(value: [u8; 64]) -> Self {
        crate::crypt::Hash::from(value).into()
    }
}
impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut data = [0u8; 64 * 2];
        // Safety: data is exactly the right size for the hex output
        unsafe {
            hex::encode_to_slice(<[u8; 64]>::from(self.0), &mut data).unwrap_unchecked();
        }
        f.write_str(super::StackStr(data).as_ref())
    }
}

impl std::ops::Deref for Hash {
    type Target = crate::crypt::Hash;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::crypt::Hash> for Hash {
    fn from(value: crate::crypt::Hash) -> Self {
        Self(value)
    }
}

// impl From<[u8; 64]> for Hash {
//     fn from(value: [u8; 64]) -> Self {
//         Into::<crate::crypt::Hash>::into(value).into()
//     }
// }
impl From<Hash> for crate::crypt::Hash {
    fn from(val: Hash) -> Self {
        val.0
    }
}

impl<'de> serde::Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        s.parse()
            .map_err(|_| serde::de::Error::custom("Invalid sha3_512 hash"))
    }
}

impl serde::Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(&self.0[..]))
    }
}

#[test]
fn test_read() {
    const TEST_DATA: &str = "d9e09f8529fed3b909876f34f21c7148d73de01d82f8aee43c52d9ee2601999ddcbf4593a19baac497d9d83bb98c94c2508b8157efafcd6484cbca7c4953af5f";
    let _hash: Hash = TEST_DATA.parse().expect("Correct Hash not read.");
    //dbg!(_hash);
    const TEST_DATA_NOPREFIX: &str = 
        "0xd9e09f8529fed3b909876f34f21c7148d73de01d82f8aee43c52d9ee2601999ddcbf4593a19baac497d9d83bb98c94c2508b8157efafcd6484cbca7c4953af5f";
    <Hash as std::str::FromStr>::from_str(TEST_DATA_NOPREFIX)
        .expect_err("Accepted data with prefix.");
    const TEST_DATA_WITH_UPPER: &str = 
        "0xd9e09f8529fed3b909876F34f21c7148d73de01d82f8aEe43c52d9ee2601999dDcbf4593a19baac497d9d83bb98c94c2508b8157efafcd6484cbca7c4953af5f";
    <Hash as std::str::FromStr>::from_str(TEST_DATA_WITH_UPPER)
        .expect_err("Accepted data witH mIxeD cAsE.");
}

#[test]
fn test_write() {
    const TEST_DATA: &str = "d9e09f8529fed3b909876f34f21c7148d73de01d82f8aee43c52d9ee2601999ddcbf4593a19baac497d9d83bb98c94c2508b8157efafcd6484cbca7c4953af5f";
    let hash_thing: Hash = TEST_DATA.parse().expect("Correct Hash not read.");
    println!("Cannot Check Output at this time.");
    assert_eq!(TEST_DATA, &hash_thing.to_string(), "stuff broke");
}
