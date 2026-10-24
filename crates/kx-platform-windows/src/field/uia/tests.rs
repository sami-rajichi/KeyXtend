//! UI Automation on hidden test edit boxes: their kinds and ids, the words before the caret in Latin and Arabic,
//! password and read-only boxes that are never read, the read cap, and focus in our own process.

use super::*;
use crate::field::run::{Job, Shared, spawn};
use crate::field::testedit::{LONG_MS, TestEdit, element, far, started};
use crate::field::win::change_of;
use crate::lock;
use crate::scroll::testwin::{WAIT, one_client};
use crate::uia::{caret, value};
use kx_module_api::Redacted;
use kx_module_api::ports::{FieldChange, FieldConfig};
use std::cell::Cell;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use windows::Win32::UI::WindowsAndMessaging::{ES_PASSWORD, ES_READONLY};

/// Latin words, the caret after the last.
const LATIN: &str = "alpha beta gamma delta";
/// Arabic words, the caret after the last.
const ARABIC: &str = "أنا أحب القراءة كثيرا";
/// The edit password style bit.
const PW: u32 = 0x0020;

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
    let read = |el: &IUIAutomationElement, n| before_cursor(el, n, far());
    assert_eq!(read(&arabic, 2), Ok(Some("القراءة كثيرا".to_owned())));
    assert_eq!(read(&arabic, 9), Ok(Some(ARABIC.to_owned())));
    assert_eq!(read(&latin, 2), Ok(Some("gamma delta".to_owned())));
    assert_eq!(read(&latin, 0), Ok(Some(String::new())));
    assert_eq!(ends_with(&latin, "delta", far()), Ok(Some(true)));
    assert_eq!(ends_with(&latin, "gamma", far()), Ok(Some(false)));
}

#[test]
fn a_password_box_is_never_read_and_a_late_read_times_out() {
    let _one = one_client();
    let (_com, auto) = started();
    let (secret_box, plain_box) = (
        TestEdit::start(LATIN, ES_PASSWORD),
        TestEdit::start(LATIN, 0),
    );
    let secret = secret_box.element(&auto);
    assert_eq!(before_cursor(&secret, 2, far()), Ok(None));
    assert_eq!(ends_with(&secret, "delta", far()), Ok(None));
    let plain = plain_box.element(&auto);
    let past = Instant::now();
    assert_eq!(before_cursor(&plain, 2, past), Err(FieldError::TimedOut));
}

#[test]
fn focus_in_our_own_process_keeps_the_last_box_and_any_other_is_told() {
    let _one = one_client();
    let (_com, auto) = started();
    let heard: Arc<Mutex<Vec<FieldChange>>> = Arc::default();
    let log = Arc::clone(&heard);
    let (give, got) = mpsc::channel();
    let body = move |inbox: &Receiver<_>, shared: &Arc<Shared>, _| {
        let _ = give.send(Arc::clone(shared));
        for _job in inbox {}
    };
    let tell = Box::new(move |change| lock(&log).push(change));
    let run = spawn(FieldConfig { read_ms: LONG_MS }, tell, body).unwrap();
    let shared = got.recv_timeout(WAIT).unwrap();
    let (plain, secret) = (
        TestEdit::start(LATIN, 0),
        TestEdit::start(LATIN, ES_PASSWORD),
    );
    assert_eq!(change_of(&plain.element(&auto), std::process::id()), None);
    let told = change_of(&secret.element(&auto), 0).unwrap();
    shared.told(told);
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
    assert_eq!(before_cursor(&fixed, 2, far()), Ok(None));
}

/// What a run told a stale `Text` reads from the box with raw handle `raw`, and how many text calls its thread made;
/// its thread and client are gone on return.
fn stale_read(raw: isize) -> (Result<Option<String>, FieldError>, usize) {
    let (give, got) = mpsc::channel();
    let (count, counted) = mpsc::channel();
    let (ended, done) = mpsc::channel();
    let body = move |inbox: &Receiver<Job>, shared: &Arc<Shared>, _| {
        let _ = give.send(Arc::clone(shared));
        let (com, auto) = started();
        let el = element(&auto, raw);
        for job in inbox {
            if let Job::Before {
                max_words,
                by,
                reply,
            } = job
            {
                let _ = reply.send(before_cursor(&el, max_words, by));
                let _ = count.send(TEXT_CALLS.with(Cell::get));
            }
        }
        drop((el, auto, com));
        let _ = ended.send(());
    };
    let mut run = spawn(
        FieldConfig { read_ms: LONG_MS },
        Box::new(|_: FieldChange| {}),
        body,
    )
    .unwrap();
    let shared: Arc<Shared> = got.recv_timeout(WAIT).unwrap();
    shared.told(FieldChange {
        kind: FieldKind::Text,
        box_id: 1,
    });
    let read = run.before_cursor(2).map(|t| t.map(Redacted::into_inner));
    let calls = counted.recv_timeout(WAIT).unwrap();
    drop(run);
    done.recv_timeout(WAIT).unwrap();
    (read, calls)
}

#[test]
fn a_stale_text_kind_never_reads_a_box_that_is_now_a_password_box() {
    let _one = one_client();
    let (secret, plain) = (
        TestEdit::start(LATIN, ES_PASSWORD),
        TestEdit::start(LATIN, 0),
    );
    assert_eq!(stale_read(secret.raw()), (Ok(None), 0));
    let (read, calls) = stale_read(plain.raw());
    assert_eq!(read, Ok(Some("gamma delta".to_owned())));
    assert!(calls > 0, "the plain box is read");
}

#[test]
#[allow(unsafe_code, reason = "Looks up a test box's text pattern.")]
fn caret_and_value_stop_at_the_read_cap() {
    let _one = one_client();
    let (_com, auto) = started();
    let long = "a ".repeat(READ_CAP);
    let (long_box, short_box) = (TestEdit::start(&long, 0), TestEdit::start(LATIN, 0));
    let (long_el, short_el) = (long_box.element(&auto), short_box.element(&auto));
    assert_eq!(value(&long_el), None);
    assert_eq!(value(&short_el), Some(LATIN.to_owned()));
    let pattern = |el: &IUIAutomationElement| {
        // SAFETY: a plain pattern lookup on a live element.
        unsafe { el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) }.unwrap()
    };
    assert!(matches!(
        caret(&pattern(&long_el)),
        Err(crate::uia::UiaError::TooLong("caret"))
    ));
    assert_eq!(caret(&pattern(&short_el)).ok(), Some(LATIN.len()));
}
