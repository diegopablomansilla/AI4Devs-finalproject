//! Typed identifier (`Id<T>`) — Crockford ULID, 26 chars, no table prefix.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<T> {
    inner: ulid::Ulid,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    pub fn generate() -> Self {
        Self {
            inner: ulid::Ulid::generate(),
            _marker: PhantomData,
        }
    }

    pub fn inner_ulid(&self) -> ulid::Ulid {
        self.inner
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner.to_string())
    }
}

impl<T> FromStr for Id<T> {
    type Err = InvalidId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(':') || !is_wire_ulid(s) {
            return Err(InvalidId);
        }
        let inner = ulid::Ulid::from_string(s).map_err(|_| InvalidId)?;
        let canonical = inner.to_string();
        if canonical != s {
            return Err(InvalidId);
        }
        Ok(Self {
            inner,
            _marker: PhantomData,
        })
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| serde::de::Error::custom("invalid id"))
    }
}

fn is_wire_ulid(s: &str) -> bool {
    if s.len() != 26 {
        return false;
    }
    s.bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'A'..=b'H' | b'J'..=b'N' | b'P'..=b'T' | b'V'..=b'Z'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    const CANONICAL_ULID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Dummy;

    type TestId = Id<Dummy>;

    #[test]
    fn id_parses_canonical_ulid() {
        let id: TestId = CANONICAL_ULID.parse().expect("valid ulid");
        assert_eq!(id.to_string(), CANONICAL_ULID);
    }

    #[test]
    fn id_roundtrip_display_and_json() {
        let id: TestId = CANONICAL_ULID.parse().unwrap();
        assert_eq!(id.to_string(), CANONICAL_ULID);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{CANONICAL_ULID}\""));
        let back: TestId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn generated_id_matches_ulid_pattern() {
        let id = TestId::generate();
        let s = id.to_string();
        assert_eq!(s.len(), 26);
        assert!(!s.contains(':'));
        assert!(is_wire_ulid(&s));
    }

    #[test]
    fn id_rejects_table_prefix() {
        assert!(
            "producto:01J9ZKA7M3Q8R2T5V9X1Z4B6D8"
                .parse::<TestId>()
                .is_err()
        );
        let json = "\"producto:01J9ZKA7M3Q8R2T5V9X1Z4B6D8\"";
        assert!(serde_json::from_str::<TestId>(json).is_err());
    }

    #[test]
    fn id_rejects_wrong_length() {
        let short = "01J9ZKA7M3Q8R2T5V9X1Z4B6";
        let long = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8X";
        assert!(short.parse::<TestId>().is_err());
        assert!(long.parse::<TestId>().is_err());
    }

    #[test]
    fn id_rejects_invalid_alphabet_and_lowercase() {
        assert!("01I9ZKA7M3Q8R2T5V9X1Z4B6D8".parse::<TestId>().is_err());
        assert!("01j9zka7m3q8r2t5v9x1z4b6d8".parse::<TestId>().is_err());
    }
}
