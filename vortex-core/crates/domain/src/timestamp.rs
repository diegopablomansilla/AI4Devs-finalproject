//! UTC timestamp — RFC 3339 with canonical `Z` suffix.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTimestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    secs: i64,
    nanos: u32,
}

impl Timestamp {
    pub fn from_unix(secs: i64, nanos: u32) -> Self {
        Self { secs, nanos }
    }

    pub fn as_unix(&self) -> (i64, u32) {
        (self.secs, self.nanos)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_rfc3339_utc(self.secs, self.nanos))
    }
}

impl FromStr for Timestamp {
    type Err = InvalidTimestamp;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_rfc3339_utc(s)
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrVisitor;

        impl<'de> serde::de::Visitor<'de> for StrVisitor {
            type Value = Timestamp;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an RFC 3339 UTC timestamp string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Timestamp::from_str(v).map_err(|_| serde::de::Error::custom("invalid timestamp"))
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

fn parse_rfc3339_utc(s: &str) -> Result<Timestamp, InvalidTimestamp> {
    if s.is_empty() {
        return Err(InvalidTimestamp);
    }
    let (date_time, tz) = split_rfc3339(s)?;
    let (secs, nanos) = parse_datetime_utc(date_time)?;
    match tz {
        Rfc3339Tz::Z => Ok(Timestamp { secs, nanos }),
        Rfc3339Tz::PlusZero => Ok(Timestamp { secs, nanos }),
        Rfc3339Tz::Other => Err(InvalidTimestamp),
    }
}

enum Rfc3339Tz {
    Z,
    PlusZero,
    Other,
}

fn split_rfc3339(s: &str) -> Result<(&str, Rfc3339Tz), InvalidTimestamp> {
    if let Some(dt) = s.strip_suffix('Z') {
        return Ok((dt, Rfc3339Tz::Z));
    }
    if let Some(idx) = s.rfind('+') {
        let (dt, off) = s.split_at(idx);
        if off == "+00:00" || off == "+0000" {
            return Ok((dt, Rfc3339Tz::PlusZero));
        }
        return Ok((dt, Rfc3339Tz::Other));
    }
    if s.contains('-')
        && s.contains('T')
        && let Some(idx) = s.rfind('-')
    {
        let prefix = &s[..idx];
        if prefix.len() >= 19 && prefix.as_bytes().get(10) == Some(&b'T') {
            return Ok((prefix, Rfc3339Tz::Other));
        }
    }
    Err(InvalidTimestamp)
}

fn parse_datetime_utc(dt: &str) -> Result<(i64, u32), InvalidTimestamp> {
    let (date, time) = dt.split_once('T').ok_or(InvalidTimestamp)?;
    let (year, month, day) = parse_date(date)?;
    let (hour, minute, second, nanos) = parse_time(time)?;
    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
    Ok((secs, nanos))
}

fn parse_date(s: &str) -> Result<(i32, u32, u32), InvalidTimestamp> {
    let bytes = s.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(InvalidTimestamp);
    }
    let year = parse_u32(&s[0..4])?;
    let month = parse_u32(&s[5..7])?;
    let day = parse_u32(&s[8..10])?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(InvalidTimestamp);
    }
    Ok((year as i32, month, day))
}

fn parse_time(s: &str) -> Result<(u32, u32, u32, u32), InvalidTimestamp> {
    let (hms, frac) = match s.split_once('.') {
        Some(pair) => pair,
        None => (s, ""),
    };
    let bytes = hms.as_bytes();
    if bytes.len() != 8 || bytes[2] != b':' || bytes[5] != b':' {
        return Err(InvalidTimestamp);
    }
    let hour = parse_u32(&hms[0..2])?;
    let minute = parse_u32(&hms[3..5])?;
    let second = parse_u32(&hms[6..8])?;
    if hour > 23 || minute > 59 || second > 59 {
        return Err(InvalidTimestamp);
    }
    let nanos = parse_fraction(frac)?;
    Ok((hour, minute, second, nanos))
}

fn parse_fraction(frac: &str) -> Result<u32, InvalidTimestamp> {
    if frac.is_empty() {
        return Ok(0);
    }
    if !frac.bytes().all(|b| b.is_ascii_digit()) || frac.len() > 9 {
        return Err(InvalidTimestamp);
    }
    let padded = format!("{:0<9}", frac);
    padded[..9].parse::<u32>().map_err(|_| InvalidTimestamp)
}

fn parse_u32(s: &str) -> Result<u32, InvalidTimestamp> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(InvalidTimestamp);
    }
    s.parse().map_err(|_| InvalidTimestamp)
}

fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y);
    let m = i64::from(m);
    let d = i64::from(d);
    let y_adj = y - (if m <= 2 { 1 } else { 0 });
    let era = y_adj / 400;
    let yoe = y_adj - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn format_rfc3339_utc(secs: i64, nanos: u32) -> String {
    let (y, m, d) = civil_from_days(secs / 86_400);
    let day_secs = secs.rem_euclid(86_400);
    let hour = (day_secs / 3_600) as u32;
    let minute = ((day_secs % 3_600) / 60) as u32;
    let second = (day_secs % 60) as u32;
    if nanos == 0 {
        format!("{y:04}-{m:02}-{d:02}T{hour:02}:{minute:02}:{second:02}Z")
    } else {
        let frac = format_nanos(nanos);
        format!("{y:04}-{m:02}-{d:02}T{hour:02}:{minute:02}:{second:02}.{frac}Z")
    }
}

fn format_nanos(nanos: u32) -> String {
    let mut s = format!("{nanos:09}");
    while s.ends_with('0') {
        s.pop();
    }
    s
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn timestamp_parses_rfc3339_z() {
        let ts: Timestamp = "2026-09-24T21:33:00Z".parse().unwrap();
        assert_eq!(ts.to_string(), "2026-09-24T21:33:00Z");
        let json = serde_json::to_string(&ts).unwrap();
        assert_eq!(json, "\"2026-09-24T21:33:00Z\"");
    }

    #[test]
    fn timestamp_normalizes_plus_zero_offset() {
        let ts: Timestamp = "2026-09-24T21:33:00+00:00".parse().unwrap();
        assert!(ts.to_string().ends_with('Z'));
        let from_z: Timestamp = "2026-09-24T21:33:00Z".parse().unwrap();
        assert_eq!(ts, from_z);
    }

    #[test]
    fn timestamp_roundtrip_display_and_json() {
        let ts: Timestamp = "2026-09-24T21:33:00Z".parse().unwrap();
        let json = serde_json::to_string(&ts).unwrap();
        assert!(json.contains('Z'));
        let back: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ts);
    }

    #[test]
    fn timestamp_rejects_naive_datetime() {
        assert!("2026-09-24T21:33:00".parse::<Timestamp>().is_err());
    }

    #[test]
    fn timestamp_rejects_non_utc_offset() {
        assert!("2026-09-24T21:33:00-03:00".parse::<Timestamp>().is_err());
    }

    #[test]
    fn timestamp_rejects_garbage_and_json_number() {
        assert!("".parse::<Timestamp>().is_err());
        assert!("2026/09/24".parse::<Timestamp>().is_err());
        assert!(serde_json::from_str::<Timestamp>("1640000000").is_err());
    }
}
