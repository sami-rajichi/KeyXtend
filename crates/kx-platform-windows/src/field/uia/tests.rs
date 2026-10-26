//! UI Automation on hidden test edit boxes: their kinds and ids, the words before the caret in Latin and Arabic,
//! password and read-only boxes that are never read, the caret range's read-only answer, the read cap in characters,
//! and which process an element shows in.

use super::*;
use crate::field::testedit::{CHARS, CONFIG, TestEdit, far, started};
use crate::field::win::WinProbe;
use crate::lock;
use crate::scroll::testwin::{WAIT, one_client};
use crate::uia::{caret, value};
use kx_module_api::Redacted;
use kx_module_api::ports::{FieldChange, FieldEnds, FieldRead, FieldRun};
use kx_platform::field::{Probe, Shared, spawn};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use windows::Win32::UI::WindowsAndMessaging::{ES_PASSWORD, ES_READONLY};

mod errors;
mod select;
mod stale;

/// Latin words, the caret after the last.
const LATIN: &str = "alpha beta gamma delta";
/// Arabic words, the caret after the last.
const ARABIC: &str = "أنا أحب القراءة كثيرا";
/// The edit password style bit, as a window style.
const PW: u32 = ES_PASSWORD.cast_unsigned();
/// A read cap that splits the last two words of both texts.
const SHORT: usize = 10;

/// What a test run heard.
type Heard = Arc<Mutex<Vec<FieldChange>>>;

/// The test reads' cap.
fn chars() -> usize {
    usize::from(CHARS)
}

/// The text of `read`, unwrapped.
fn text(read: Result<FieldRead, FieldError>) -> Result<Option<String>, FieldError> {
    read.map(|r| r.text.map(Redacted::into_inner))
}

/// A read that found a box of `kind` and no text.
fn closed(kind: FieldKind) -> FieldRead {
    FieldRead { kind, text: None }
}

#[test]
fn a_plain_box_is_text_and_an_edit_box_with_the_password_style_is_a_password_box() {
    let _one = one_client();
    let (_com, auto) = started();
    let (plain, secret) = (
        TestEdit::start(LATIN, 0),
        TestEdit::start(LATIN, ES_PASSWORD),
    );
    assert_eq!(kind_of(&plain.element(&auto)), FieldKind::Text);
    let facts = facts(&secret.element(&auto));
    assert_eq!((facts.class.as_str(), facts.style & PW), ("Edit", PW));
    assert_eq!(kind_of(&secret.element(&auto)), FieldKind::Password);
}

#[test]
fn each_box_has_its_own_id_that_stays() {
    let _one = one_client();
    let (_com, auto) = started();
    let (one, two) = (TestEdit::start(LATIN, 0), TestEdit::start(LATIN, 0));
    let id = box_id(&one.element(&auto));
    assert_ne!(id, 0);
    assert_eq!(box_id(&one.element(&auto)), id);
    assert_ne!(box_id(&two.element(&auto)), id);
}

#[test]
fn the_words_before_the_caret_are_cut_to_the_last_n_in_latin_and_arabic() {
    let _one = one_client();
    let (_com, auto) = started();
    let (latin_box, arabic_box) = (TestEdit::start(LATIN, 0), TestEdit::start(ARABIC, 0));
    let (latin, arabic) = (latin_box.element(&auto), arabic_box.element(&auto));
    let read = |el: &IUIAutomationElement, n| text(before_cursor(el, n, far(), chars()));
    assert_eq!(read(&arabic, 2), Ok(Some("القراءة كثيرا".to_owned())));
    assert_eq!(read(&arabic, 9), Ok(Some(ARABIC.to_owned())));
    assert_eq!(read(&latin, 2), Ok(Some("gamma delta".to_owned())));
    assert_eq!(read(&latin, 0), Ok(Some(String::new())));
    let ended = |ends| {
        Ok(FieldEnds {
            kind: FieldKind::Text,
            ends,
        })
    };
    assert_eq!(
        ends_with(&latin, "delta", far(), chars()),
        ended(Some(true))
    );
    assert_eq!(
        ends_with(&latin, "gamma", far(), chars()),
        ended(Some(false))
    );
}

#[test]
fn a_read_past_the_cap_in_characters_keeps_only_the_whole_words_inside_it() {
    let _one = one_client();
    let (_com, auto) = started();
    let (latin_box, arabic_box) = (TestEdit::start(LATIN, 0), TestEdit::start(ARABIC, 0));
    let (latin, arabic) = (latin_box.element(&auto), arabic_box.element(&auto));
    let read = |el: &IUIAutomationElement| text(before_cursor(el, 2, far(), SHORT));
    assert_eq!(read(&latin), Ok(Some("delta".to_owned())));
    assert_eq!(read(&arabic), Ok(Some("كثيرا".to_owned())));
    let long = "a".repeat(SHORT + 1);
    let unsure = FieldEnds {
        kind: FieldKind::Text,
        ends: None,
    };
    assert_eq!(ends_with(&latin, &long, far(), SHORT), Ok(unsure));
}

#[test]
fn a_password_box_is_never_read_but_reported_and_a_late_read_times_out() {
    let _one = one_client();
    let (_com, auto) = started();
    let (secret_box, plain_box) = (
        TestEdit::start(LATIN, ES_PASSWORD),
        TestEdit::start(LATIN, 0),
    );
    let secret = secret_box.element(&auto);
    let read = before_cursor(&secret, 2, far(), chars());
    assert_eq!(
        read,
        Ok(closed(FieldKind::Password)),
        "the read tells the kind it saw"
    );
    let no = FieldEnds {
        kind: FieldKind::Password,
        ends: Some(false),
    };
    assert_eq!(
        ends_with(&secret, "delta", far(), chars()),
        Ok(no),
        "a box that is no text box never ends with a word"
    );
    let plain = plain_box.element(&auto);
    let past = Instant::now();
    let late = before_cursor(&plain, 2, past, chars());
    assert_eq!(late, Err(FieldError::TimedOut));
}

/// A run with an idle thread, its shared box, and what its `tell` heard.
fn idle() -> (Box<dyn FieldRun>, Arc<Shared>, Heard) {
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let (give, got) = mpsc::channel();
    let body = move |inbox: &Receiver<_>, shared: &Arc<Shared>, _| {
        let _ = give.send(Arc::clone(shared));
        for _job in inbox {}
    };
    let tell = Box::new(move |change| lock(&log).push(change));
    let run = spawn(CONFIG, tell, body).unwrap();
    (run, got.recv_timeout(WAIT).unwrap(), heard)
}

#[test]
fn the_probe_tells_our_own_elements_and_the_box_of_any_other() {
    let _one = one_client();
    let (_com, auto) = started();
    let (run, shared, heard) = idle();
    let (plain, secret) = (
        TestEdit::start(LATIN, 0),
        TestEdit::start(LATIN, ES_PASSWORD),
    );
    let ours = WinProbe::new();
    assert!(ours.ours(&plain.element(&auto)), "the test boxes are ours");
    shared.moved(&ours, &plain.element(&auto));
    let elsewhere = WinProbe { me: 0 };
    assert!(!elsewhere.ours(&secret.element(&auto)));
    shared.moved(&elsewhere, &secret.element(&auto));
    let want = FieldChange {
        kind: FieldKind::Password,
        box_id: box_id(&secret.element(&auto)),
    };
    assert_eq!(*lock(&heard), [want]);
    assert_eq!(run.kind(), FieldKind::Password);
}

#[test]
fn a_read_only_box_is_unknown_and_never_read() {
    let _one = one_client();
    let (_com, auto) = started();
    let fixed_box = TestEdit::start(LATIN, ES_READONLY);
    let fixed = fixed_box.element(&auto);
    assert!(facts(&fixed).read_only);
    assert_eq!(kind_of(&fixed), FieldKind::Unknown);
    assert_eq!(
        before_cursor(&fixed, 2, far(), chars()),
        Ok(closed(FieldKind::Unknown))
    );
}

#[test]
fn the_caret_range_alone_tells_a_read_only_text() {
    let _one = one_client();
    let (_com, auto) = started();
    let (fixed_box, plain_box) = (
        TestEdit::start(LATIN, ES_READONLY),
        TestEdit::start(LATIN, 0),
    );
    let range = |edit: &TestEdit| caret_of(&edit.element(&auto)).unwrap().unwrap().range;
    assert!(read_only_range(&range(&fixed_box)));
    assert!(!read_only_range(&range(&plain_box)));
}

#[test]
#[allow(unsafe_code, reason = "Looks up a test box's text pattern.")]
fn caret_and_value_stop_at_their_cap() {
    let _one = one_client();
    let (_com, auto) = started();
    let cap = chars();
    let long = "a ".repeat(cap);
    let (long_box, short_box) = (TestEdit::start(&long, 0), TestEdit::start(LATIN, 0));
    let (long_el, short_el) = (long_box.element(&auto), short_box.element(&auto));
    assert_eq!(value(&long_el, cap), None);
    assert_eq!(value(&short_el, cap), Some(LATIN.to_owned()));
    let pattern = |el: &IUIAutomationElement| {
        // SAFETY: a plain pattern lookup on a live element.
        unsafe { el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) }.unwrap()
    };
    assert!(matches!(
        caret(&pattern(&long_el), cap),
        Err(crate::uia::UiaError::TooLong("caret"))
    ));
    assert_eq!(caret(&pattern(&short_el), cap).ok(), Some(LATIN.len()));
}
