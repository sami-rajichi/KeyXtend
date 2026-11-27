//! The shipped table parses with every value in place, and a wrong one is refused.

use super::*;

/// The shipped text with `from` written as `to`, once.
fn changed(from: &str, to: &str) -> String {
    assert!(TEXT.contains(from), "{from}");
    TEXT.replacen(from, to, 1)
}

#[test]
fn the_shipped_table_parses_and_names_media_foundation() {
    let t = table().unwrap();
    assert!(t.media_dlls.iter().any(|d| d == "mfplat.dll"));
    assert!(t.sound.cushion_ms < t.sound.queue_ms);
    for channels in [1, 2, 4, 6, 8] {
        let rows = t.gains(channels).unwrap();
        assert_eq!(rows.len(), usize::from(channels));
    }
    assert_eq!(t.gains(3), None);
}

#[test]
fn a_zero_value_or_an_empty_list_is_refused() {
    for (from, to) in [
        ("bits_s = 8_000_000", "bits_s = 0"),
        ("buffers = 2", "buffers = 0"),
        ("cushion_ms = 100", "cushion_ms = 0"),
        (
            r#"media_dlls = ["mfplat.dll", "mfreadwrite.dll"]"#,
            "media_dlls = []",
        ),
        (r#"["mfplat.dll","#, r#"[" ","#),
    ] {
        assert_eq!(
            Table::parse(&changed(from, to)).err(),
            Some(TableError::Empty)
        );
    }
}

#[test]
fn sound_times_out_of_order_are_refused() {
    for (from, to) in [
        ("cushion_ms = 100", "cushion_ms = 1_000"),
        ("drift_ms = 20", "drift_ms = 100"),
    ] {
        assert_eq!(
            Table::parse(&changed(from, to)).err(),
            Some(TableError::Order)
        );
    }
}

#[test]
fn a_wrong_downmix_row_is_refused() {
    for (from, to) in [
        ("1 = [[1.0, 1.0]]", "3 = [[1.0, 1.0]]"),
        ("1 = [[1.0, 1.0]]", "one = [[1.0, 1.0]]"),
        ("1 = [[1.0, 1.0]]", "1 = [[1.0, -1.0]]"),
        ("1 = [[1.0, 1.0]]", "1 = [[1.0, nan]]"),
    ] {
        let got = Table::parse(&changed(from, to)).err();
        assert_eq!(got, Some(TableError::Downmix), "{to}");
    }
}

#[test]
fn an_unknown_key_or_a_wrong_shape_does_not_parse() {
    for text in [
        changed("[video]", "[video]\nspeed = 1"),
        changed("bits_s = 8_000_000", "bits_s = \"fast\""),
        String::new(),
    ] {
        let got = Table::parse(&text).err();
        assert!(matches!(got, Some(TableError::Parse(_))), "{got:?}");
    }
}
