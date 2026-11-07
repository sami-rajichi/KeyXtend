//! Paste: breaks at the ends go, an interior one becomes one space in a shown field and refuses a hidden one, in a
//! wiped buffer; only the take waited for counts, until its wait runs out (a fake clock, no sleep).

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{ClipTaken, ClipText};

use super::*;

/// The default wait.
const WAIT: Duration = Duration::from_millis(3000);
/// The field's characters.
const FIELD: usize = 40;
/// When each test starts.
const T0: Mono = Mono::from_us(1_000_000);
/// The breaks past CR and LF: NEL and the Unicode line and paragraph separators.
const SEPARATORS: [char; 3] = ['\u{85}', '\u{2028}', '\u{2029}'];

/// The answer to round `round`, which found nothing.
fn taken(round: u32) -> ClipTaken {
    ClipTaken {
        round,
        text: ClipText::Empty,
    }
}

/// What `copied` becomes in a field that is `hidden` or not.
fn put(copied: &str, hidden: bool) -> Result<String, FillNote> {
    flat(copied, hidden).map(|t| t.to_string())
}

#[test]
fn each_interior_break_of_every_kind_becomes_one_space_in_a_shown_field() {
    for (copied, want) in [
        ("a\rb", "a b"),
        ("a\nb", "a b"),
        ("a\r\nb", "a b"),
        ("a\u{2028}b", "a b"),
        ("a\u{2029}b", "a b"),
        ("a\u{85}b", "a b"),
        ("a\n\nb", "a  b"),
        ("a\n\rb", "a  b"),
        ("كلمة\r\nسر", "كلمة سر"),
        ("plain", "plain"),
    ] {
        assert_eq!(put(copied, false).as_deref(), Ok(want), "{copied:?}");
    }
}

#[test]
fn breaks_at_both_ends_are_trimmed_so_no_space_hides_in_a_value() {
    for hidden in [false, true] {
        assert_eq!(put("pw\r\n", hidden).as_deref(), Ok("pw"), "{hidden}");
        assert_eq!(put("\npw", hidden).as_deref(), Ok("pw"), "{hidden}");
        for sep in SEPARATORS {
            assert_eq!(put(&format!("{sep}pw{sep}"), hidden).as_deref(), Ok("pw"));
        }
    }
    assert_eq!(put("a\r\nb\n", false).as_deref(), Ok("a b"));
}

#[test]
fn an_interior_break_refuses_the_paste_in_a_hidden_field() {
    assert_eq!(put("a\r\nb\n", true), Err(FillNote::Refused));
    assert_eq!(put("user\npassword", true), Err(FillNote::Refused));
    for sep in SEPARATORS {
        assert_eq!(put(&format!("a{sep}b"), true), Err(FillNote::Refused));
    }
}

#[test]
fn a_text_of_only_breaks_or_none_is_no_text() {
    for hidden in [false, true] {
        assert_eq!(put("\r\n", hidden), Err(FillNote::NoText));
        assert_eq!(put("", hidden), Err(FillNote::NoText));
        for sep in SEPARATORS {
            assert_eq!(put(&format!("{sep}{sep}"), hidden), Err(FillNote::NoText));
        }
    }
}

#[test]
fn other_hidden_marks_stay_for_the_field_rules_to_refuse() {
    assert_eq!(put("a\tb\u{200b}c", false).as_deref(), Ok("a\tb\u{200b}c"));
}

#[test]
fn the_flat_text_fits_the_room_it_starts_with() {
    let copied = "x\u{2028}".repeat(50);
    let made = flat(&copied, false).unwrap();
    assert!(made.capacity() >= made.len() && made.capacity() <= copied.len());
}

#[test]
fn a_take_asks_for_twice_the_fields_characters_and_only_the_one_waited_for_counts() {
    let mut pasting = Pasting::new(WAIT, FIELD);
    let first = pasting.ask(T0);
    let second = pasting.ask(T0);
    assert_ne!(first.round, second.round);
    assert_eq!(second, ClipTake::new(second.round, FIELD));
    assert!(!pasting.answers(&taken(first.round)));
    assert!(pasting.answers(&taken(second.round)));
    assert!(!pasting.answers(&taken(second.round)), "once");
}

#[test]
fn the_wait_runs_out_once_and_a_late_answer_is_dropped() {
    let mut pasting = Pasting::new(WAIT, FIELD);
    assert_eq!(pasting.due(), None);
    let ask = pasting.ask(T0);
    let due = T0.saturating_add(WAIT);
    assert_eq!(pasting.due(), Some(due));
    assert!(!pasting.expired(T0.saturating_add(WAIT.saturating_sub(Duration::from_millis(1)))));
    assert!(pasting.expired(due));
    assert!(!pasting.expired(due), "once");
    assert_eq!(pasting.due(), None);
    assert!(!pasting.answers(&taken(ask.round)));
}

#[test]
fn a_stop_drops_the_answer_and_the_wait() {
    let mut pasting = Pasting::new(WAIT, FIELD);
    let ask = pasting.ask(T0);
    pasting.stop();
    assert_eq!(pasting.due(), None);
    assert!(!pasting.answers(&taken(ask.round)));
}
