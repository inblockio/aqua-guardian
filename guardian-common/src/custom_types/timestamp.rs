const FORMAT: &str = "%Y%m%d%H%M%S";

#[derive(Debug, Clone, Default)]
/// Used to check for witness events.
pub struct Timestamp(chrono::NaiveDateTime);

impl From<chrono::NaiveDateTime> for Timestamp {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Timestamp(value)
    }
}

impl From<Timestamp> for chrono::NaiveDateTime {
    fn from(val: Timestamp) -> Self {
        val.0
    }
}

pub fn format_time_stamp(
    timestamp: &chrono::NaiveDateTime,
) -> chrono::format::DelayedFormat<chrono::format::StrftimeItems<'static>> {
    timestamp.format(FORMAT)
}

impl std::str::FromStr for Timestamp {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.chars().all(|c| c.is_numeric() && c.is_ascii()) {
            //Throw error, as it doesn't natively
            return chrono::NaiveDateTime::parse_from_str("?", "-").map(Timestamp);
        }
        chrono::NaiveDateTime::parse_from_str(s, FORMAT).map(Timestamp)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_time_stamp(&self.0).fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        s.parse()
            .map_err(|_| serde::de::Error::custom("Invalid timestamp"))
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, serialize: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize.serialize_str(&format_time_stamp(&self.0).to_string())
    }
}

#[test]
fn read_time() {
    const TEST_DATA: &str = "20240525000123";
    let _timestamp: Timestamp = TEST_DATA.parse().expect("Timestamp was read incorrectly.");
    //dbg!(_timestamp);
    const TEST_DATA_WITH_WHITESPACE: &str = "2024 05 25 12 30 00";
    let whitestamp: Result<Timestamp, _> = TEST_DATA_WITH_WHITESPACE.parse();
    whitestamp.expect_err("Whitespace was not rejected.");
}

#[test]
fn write_time() {
    const TEST_DATA: &str = "20240525000123";
    let timestamp_thing: Timestamp = TEST_DATA.parse().expect("Timestamp was read incorrectly.");
    //dbg!(timestamp_thing);
    assert_eq!(TEST_DATA,timestamp_thing.to_string(),"Learn to read the time again.")
}
