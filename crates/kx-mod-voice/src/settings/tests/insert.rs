//! How a dictated text goes in (owner, 2026-11-17): typed to start, or pasted at once, any other name refused, and the
//! wait for the clipboard history's answer within its range.

use super::*;
use std::time::Duration;

#[test]
fn a_text_is_typed_to_start_and_waits_paste_ms_for_a_paste() {
    let s = VoiceSettings::from_defaults().unwrap();
    assert_eq!((s.insert.as_str(), s.way()), ("type", Insert::Type));
    assert_eq!(s.paste_ms, 2_000);
    assert_eq!(s.paste_wait(), Duration::from_millis(2_000));
    assert_eq!(PASTE_MS, 500..=10_000);
}

#[test]
fn insert_is_type_or_paste_and_any_other_name_is_refused() {
    for (name, way) in INSERTS {
        let s = parse_as::<VoiceSettings>(&with("insert", name)).unwrap();
        assert_eq!(s.way(), way, "{name}");
    }
    assert_eq!(INSERTS.map(|(n, _)| n), ["type", "paste"]);
    for bad in ["", "Paste", "copy", "type "] {
        assert!(refused(&with("insert", bad), "insert"), "{bad:?}");
    }
    assert!(unread(&with("insert", 1)));
}

#[test]
fn paste_ms_takes_its_ends_and_refuses_past_them() {
    let (lo, hi) = ends(&PASTE_MS);
    assert!(taken(&with("paste_ms", lo)) && taken(&with("paste_ms", hi)));
    assert!(refused(&with("paste_ms", lo - 1), "paste_ms"));
    assert!(refused(&with("paste_ms", hi + 1), "paste_ms"));
}
