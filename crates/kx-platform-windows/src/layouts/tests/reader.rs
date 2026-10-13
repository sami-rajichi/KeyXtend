//! Tests of the `ToUnicodeEx` answer reader: the visible text and the dead-key sign.

use proptest::prelude::*;

use crate::layouts::{is_printable, read_typed};

#[test]
fn a_windows_answer_gives_the_text_and_whether_the_key_is_dead() {
    let buf: Vec<u16> = "^x".encode_utf16().collect();
    assert_eq!(read_typed(-1, &buf), ("^".to_string(), true), "a dead key");
    assert_eq!(read_typed(2, &buf), ("^x".to_string(), false));
    assert_eq!(read_typed(0, &buf), (String::new(), false), "types nothing");
    let bs: Vec<u16> = "\u{8}".encode_utf16().collect();
    assert_eq!(read_typed(1, &bs), (String::new(), false), "a control");
    assert_eq!(read_typed(1, &[0]), (String::new(), false), "U+0000");
    assert_eq!(read_typed(9, &bs).0, "", "a count past the buffer");
    assert_eq!(read_typed(i32::MIN, &buf), ("^x".to_string(), true));
}

proptest! {
    #[test]
    fn any_answer_reads_inside_its_buffer(
        n: i32,
        buf in prop::collection::vec(any::<u16>(), 0..40),
    ) {
        let (text, dead) = read_typed(n, &buf);
        let taken = usize::try_from(n.unsigned_abs()).unwrap_or(usize::MAX);
        prop_assert!(text.encode_utf16().count() <= taken.min(buf.len()));
        prop_assert_eq!(dead, n < 0);
    }

    #[test]
    fn a_visible_text_reads_back_live_or_dead(text in "[a-zA-Z\u{621}-\u{64A}]{1,8}") {
        let buf: Vec<u16> = text.encode_utf16().collect();
        let n = i32::try_from(buf.len()).unwrap();
        prop_assert_eq!(read_typed(n, &buf), (text.clone(), false));
        prop_assert_eq!(read_typed(-n, &buf), (text, true));
    }
}

#[test]
fn printable_means_a_visible_character() {
    for (text, want) in [
        ("\r", false),
        (" ", false),
        ("\u{1b}", false),
        ("", false),
        ("a", true),
        ("ض", true),
        ("\u{64e}", true),
    ] {
        assert_eq!(is_printable(text), want, "{text:?}");
    }
}
