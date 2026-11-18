//! Opaque cursor pagination (ULID in base64url) for admin list endpoints (T-00-21).

use std::str::FromStr;

use domain::{DomainError, FieldError, Id, InvalidInputCode};

pub const DEFAULT_PAGE_LIMIT: u32 = 50;
pub const MAX_PAGE_LIMIT: u32 = 100;

const B64URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Wire page: `items` plus opaque `next_cursor` (`None` = no more pages).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

/// Encodes a typed id as an opaque cursor (base64url of the canonical 26-char ULID).
pub fn encode_cursor<T>(id: &Id<T>) -> String {
    b64url_encode_no_pad(id.to_string().as_bytes())
}

/// Decodes a query cursor. Absent/`None` means the first page.
pub fn decode_cursor<T>(cursor: Option<&str>) -> Result<Option<Id<T>>, DomainError> {
    let Some(raw) = cursor else {
        return Ok(None);
    };
    if raw.is_empty() {
        return Err(invalid_cursor());
    }
    if raw.bytes().any(|b| b == b'=' || b == b'+' || b == b'/') {
        return Err(invalid_cursor());
    }
    let decoded = b64url_decode_no_pad(raw).map_err(|_| invalid_cursor())?;
    let wire = std::str::from_utf8(&decoded).map_err(|_| invalid_cursor())?;
    let id = Id::<T>::from_str(wire).map_err(|_| invalid_cursor())?;
    Ok(Some(id))
}

/// Resolves `limit` query param: default 50, accepted range 1..=100.
pub fn resolve_limit(limit: Option<u32>) -> Result<u32, DomainError> {
    match limit {
        None => Ok(DEFAULT_PAGE_LIMIT),
        Some(n) if (1..=MAX_PAGE_LIMIT).contains(&n) => Ok(n),
        Some(_) => Err(invalid_limit()),
    }
}

/// Splits `items` (already ordered, at most `limit + 1` elements) into a page.
pub fn paginate<T, F>(items: Vec<T>, limit: u32, id_of: F) -> CursorPage<T>
where
    F: Fn(&T) -> String,
{
    let limit = limit as usize;
    if items.len() > limit {
        let next = id_of(&items[limit - 1]);
        CursorPage {
            items: items.into_iter().take(limit).collect(),
            next_cursor: Some(next),
        }
    } else {
        CursorPage {
            items,
            next_cursor: None,
        }
    }
}

fn invalid_cursor() -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![
            FieldError::try_new("cursor", "must be a valid opaque cursor")
                .expect("non-empty field message"),
        ],
    )
}

fn invalid_limit() -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![
            FieldError::try_new("limit", "must be between 1 and 100")
                .expect("non-empty field message"),
        ],
    )
}

fn b64url_encode_no_pad(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() * 4).div_ceil(3));
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = (u32::from(data[i]) << 16) | (u32::from(data[i + 1]) << 8) | u32::from(data[i + 2]);
        push_b64url_quad(&mut out, n);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = u32::from(data[i]) << 16;
        out.push(B64URL[(n >> 18) as usize] as char);
        out.push(B64URL[((n >> 12) & 63) as usize] as char);
    } else if rem == 2 {
        let n = (u32::from(data[i]) << 16) | (u32::from(data[i + 1]) << 8);
        out.push(B64URL[(n >> 18) as usize] as char);
        out.push(B64URL[((n >> 12) & 63) as usize] as char);
        out.push(B64URL[((n >> 6) & 63) as usize] as char);
    }
    out
}

fn push_b64url_quad(out: &mut String, n: u32) {
    out.push(B64URL[(n >> 18) as usize] as char);
    out.push(B64URL[((n >> 12) & 63) as usize] as char);
    out.push(B64URL[((n >> 6) & 63) as usize] as char);
    out.push(B64URL[(n & 63) as usize] as char);
}

fn b64url_decode_no_pad(input: &str) -> Result<Vec<u8>, ()> {
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let remaining = bytes.len() - i;
        if remaining >= 4 {
            let n = quad_value(&bytes[i..i + 4])?;
            out.push((n >> 16) as u8);
            out.push(((n >> 8) & 0xff) as u8);
            out.push((n & 0xff) as u8);
            i += 4;
        } else if remaining == 3 {
            let n = quad_value(&[bytes[i], bytes[i + 1], bytes[i + 2], b'A'])? >> 8;
            out.push((n >> 8) as u8);
            out.push((n & 0xff) as u8);
            i += 3;
        } else if remaining == 2 {
            let n = quad_value(&[bytes[i], bytes[i + 1], b'A', b'A'])? >> 16;
            out.push(n as u8);
            i += 2;
        } else {
            return Err(());
        }
    }
    Ok(out)
}

fn quad_value(chunk: &[u8]) -> Result<u32, ()> {
    let mut n = 0u32;
    for &b in chunk {
        let idx = b64url_index(b).ok_or(())?;
        n = (n << 6) | idx;
    }
    Ok(n)
}

fn b64url_index(b: u8) -> Option<u32> {
    match b {
        b'A'..=b'Z' => Some(u32::from(b - b'A')),
        b'a'..=b'z' => Some(u32::from(b - b'a' + 26)),
        b'0'..=b'9' => Some(u32::from(b - b'0' + 52)),
        b'-' => Some(62),
        b'_' => Some(63),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiError;
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;
    use domain::Producto;

    const CANONICAL_ULID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

    type TestId = Id<Producto>;

    fn id(s: &str) -> TestId {
        s.parse().expect("ulid")
    }

    #[test]
    fn pagination_encode_decode_roundtrip() {
        let ulid = id(CANONICAL_ULID);
        let cursor = encode_cursor(&ulid);
        assert_ne!(cursor, CANONICAL_ULID);
        assert!(!cursor.contains('='));
        assert!(
            cursor
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        let back = decode_cursor::<Producto>(Some(&cursor)).expect("decode");
        assert_eq!(back, Some(ulid));
    }

    #[test]
    fn pagination_absent_cursor_is_first_page() {
        assert_eq!(decode_cursor::<Producto>(None).expect("ok"), None);
    }

    #[test]
    fn pagination_limit_default_is_50() {
        assert_eq!(resolve_limit(None).expect("ok"), 50);
    }

    #[test]
    fn pagination_limit_1_and_100_are_accepted() {
        assert_eq!(resolve_limit(Some(1)).expect("ok"), 1);
        assert_eq!(resolve_limit(Some(100)).expect("ok"), 100);
    }

    #[test]
    fn pagination_next_cursor_when_more_remain() {
        let a = id("01J9ZKA7M3Q8R2T5V9X1Z4B6D0");
        let b = id("01J9ZKA7M3Q8R2T5V9X1Z4B6D1");
        let c = id("01J9ZKA7M3Q8R2T5V9X1Z4B6D2");
        let b_expected = b.clone();
        let page = paginate(vec![a, b, c], 2, |x| encode_cursor(x));
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[1], b_expected);
        let next = page.next_cursor.expect("next");
        assert_eq!(
            decode_cursor::<Producto>(Some(&next)).expect("decode"),
            Some(b_expected)
        );
    }

    #[test]
    fn pagination_next_cursor_null_on_last_page() {
        let a = id("01J9ZKA7M3Q8R2T5V9X1Z4B6D0");
        let b = id("01J9ZKA7M3Q8R2T5V9X1Z4B6D1");
        let page = paginate(vec![a, b], 2, |x| encode_cursor(x));
        assert_eq!(page.items.len(), 2);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn pagination_next_cursor_null_on_empty_page() {
        let page: CursorPage<TestId> =
            paginate(Vec::<TestId>::new(), 2, |x: &TestId| encode_cursor(x));
        assert!(page.items.is_empty());
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn pagination_invalid_cursor_not_base64url_is_invalid_input() {
        let err = decode_cursor::<Producto>(Some("%%%")).unwrap_err();
        assert_field_error(ApiError::from(err), "cursor");
    }

    #[test]
    fn pagination_invalid_cursor_padding_or_std_alphabet_is_invalid_input() {
        let padded = format!("{}=", encode_cursor(&id(CANONICAL_ULID)));
        let err = decode_cursor::<Producto>(Some(&padded)).unwrap_err();
        assert_field_error(ApiError::from(err), "cursor");

        let err = decode_cursor::<Producto>(Some("abcd+efgh")).unwrap_err();
        assert_field_error(ApiError::from(err), "cursor");
    }

    #[test]
    fn pagination_invalid_cursor_non_ulid_is_invalid_input() {
        let cursor = b64url_encode_no_pad(b"not-a-ulid");
        let err = decode_cursor::<Producto>(Some(&cursor)).unwrap_err();
        assert_field_error(ApiError::from(err), "cursor");

        let cursor = b64url_encode_no_pad(b"01j9zka7m3q8r2t5v9x1z4b6d8");
        let err = decode_cursor::<Producto>(Some(&cursor)).unwrap_err();
        assert_field_error(ApiError::from(err), "cursor");
    }

    #[test]
    fn pagination_empty_cursor_is_invalid_input() {
        let err = decode_cursor::<Producto>(Some("")).unwrap_err();
        assert_field_error(ApiError::from(err), "cursor");
    }

    #[test]
    fn pagination_limit_zero_is_invalid_input() {
        let err = resolve_limit(Some(0)).unwrap_err();
        assert_field_error(ApiError::from(err), "limit");
    }

    #[test]
    fn pagination_limit_101_is_invalid_input() {
        let err = resolve_limit(Some(101)).unwrap_err();
        assert_field_error(ApiError::from(err), "limit");
    }

    #[tokio::test]
    async fn pagination_invalid_cursor_maps_to_problem_json() {
        let err = ApiError::from(decode_cursor::<Producto>(Some("%%%")).unwrap_err());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            resp.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/problem+json")
        );
        let body = axum::body::to_bytes(resp.into_body(), 10_000)
            .await
            .expect("body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["code"], "invalid_input");
        let fields: Vec<_> = json["errors"]
            .as_array()
            .expect("errors")
            .iter()
            .filter_map(|e| e.get("field").and_then(|f| f.as_str()))
            .collect();
        assert!(fields.contains(&"cursor"));
    }

    fn field_in_error(err: ApiError, field: &str) -> bool {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let resp = err.into_response();
            let body = axum::body::to_bytes(resp.into_body(), 10_000)
                .await
                .expect("body");
            let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
            json["errors"]
                .as_array()
                .expect("errors")
                .iter()
                .any(|e| e.get("field").and_then(|f| f.as_str()) == Some(field))
        })
    }

    fn assert_field_error(err: ApiError, field: &str) {
        assert!(field_in_error(err.clone(), field));
        assert_eq!(err.into_response().status(), StatusCode::BAD_REQUEST);
    }
}
