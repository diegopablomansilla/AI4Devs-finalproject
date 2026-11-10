//! Signed decimal newtype — OpenAPI `Decimal` as string.

use std::fmt;
use std::str::FromStr;

use rust_decimal::Decimal as InnerDecimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(InnerDecimal);

impl Decimal {
    pub fn inner(&self) -> InnerDecimal {
        self.0
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_decimal_wire(self.0))
    }
}

impl FromStr for Decimal {
    type Err = InvalidDecimal;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_wire_decimal(s) {
            return Err(InvalidDecimal);
        }
        let inner = InnerDecimal::from_str(s).map_err(|_| InvalidDecimal)?;
        Ok(Self(inner))
    }
}

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrVisitor;

        impl<'de> serde::de::Visitor<'de> for StrVisitor {
            type Value = Decimal;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a decimal string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Decimal::from_str(v).map_err(|_| serde::de::Error::custom("invalid decimal"))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(StrVisitor)
    }
}

fn is_wire_decimal(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0usize;
    if bytes[i] == b'-' {
        i += 1;
        if i >= bytes.len() {
            return false;
        }
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_digits = i - int_start;
    if int_digits == 0 || int_digits > 12 {
        return false;
    }
    if i == bytes.len() {
        return true;
    }
    if bytes[i] != b'.' {
        return false;
    }
    i += 1;
    let frac_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let frac_digits = i - frac_start;
    i == bytes.len() && frac_digits > 0 && frac_digits <= 4
}

fn format_decimal_wire(value: InnerDecimal) -> String {
    let normalized = value.normalize();
    let s = normalized.to_string();
    if is_wire_decimal(&s) {
        return s;
    }
    if let Some((int_part, frac_part)) = s.split_once('.') {
        let frac_trimmed = frac_part.trim_end_matches('0');
        if frac_trimmed.is_empty() {
            let candidate = int_part.to_string();
            if is_wire_decimal(&candidate) {
                return candidate;
            }
        } else if frac_trimmed.len() <= 4 {
            let candidate = format!("{int_part}.{frac_trimmed}");
            if is_wire_decimal(&candidate) {
                return candidate;
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    fn assert_parses(s: &str) {
        let d: Decimal = s.parse().expect("parse");
        let displayed = d.to_string();
        assert!(
            is_wire_decimal(&displayed),
            "display {displayed} must match wire pattern"
        );
    }

    #[test]
    fn decimal_parses_contract_examples() {
        assert_parses("120.50");
        assert_parses("-3");
        assert_parses("34");
    }

    #[test]
    fn decimal_roundtrip_display_and_json() {
        let d: Decimal = "120.50".parse().unwrap();
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.starts_with('"') && json.ends_with('"'));
        let back: Decimal = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);
        let again: Decimal = back.to_string().parse().unwrap();
        assert_eq!(again, d);
    }

    #[test]
    fn decimal_rejects_json_number() {
        let json = "120.5";
        assert!(serde_json::from_str::<Decimal>(json).is_err());
    }

    #[test]
    fn decimal_rejects_invalid_strings() {
        for bad in [
            "abc",
            "1e2",
            "+120.50",
            ".5",
            "120.50000",
            "1234567890123",
            "",
        ] {
            assert!(bad.parse::<Decimal>().is_err(), "expected reject: {bad}");
            let json = format!("\"{bad}\"");
            assert!(
                serde_json::from_str::<Decimal>(&json).is_err(),
                "expected json reject: {bad}"
            );
        }
    }
}
