//! Gemini's error details read item by item: an item of another shape, or a null list, loses only itself.

use super::*;

/// `body` with `odd` items before its first detail.
fn with_odd(body: &[u8], odd: &str) -> Vec<u8> {
    let first = r#""details":["#;
    let odd = format!("{first}{odd},");
    String::from_utf8_lossy(body)
        .replace(first, &odd)
        .into_bytes()
}

#[test]
fn an_odd_detail_loses_only_itself() {
    let key = with_odd(KEY_BAD, r#""odd",null,7"#);
    assert_eq!(read(400, &[], &key), Got::Fault(Key));
    let day = with_odd(QUOTA_DAY, r#"{"violations":null},[]"#);
    assert_eq!(read(BUSY, &[], &day), used(None));
}
