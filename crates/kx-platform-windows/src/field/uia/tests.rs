//! UI Automation on hidden test edit boxes: their kinds and ids, the words before the caret in Latin and Arabic, a
//! password box that is never read, and focus in our own process.

use super::*;
use crate::field::run::{Shared, spawn};
use crate::field::testedit::{LONG_MS, TestEdit, far, started};
use crate::field::win::seen;
use crate::lock;
use crate::scroll::testwin::{WAIT, one_client};
use kx_module_api::ports::{FieldChange, FieldConfig};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

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
    let (plain, secret) = (TestEdit::start(LATIN, false), TestEdit::start(LATIN, true));
    assert_eq!(kind_of(&plain.element(&auto)), FieldKind::Text);
    let facts = facts(&secret.element(&auto));
    assert_eq!((facts.class.as_str(), facts.style & PW), ("Edit", PW));
    assert_eq!(kind_of(&secret.element(&auto)), FieldKind::Password);
}

#[test]
fn each_box_has_its_own_id_that_stays() {
    let _one = one_client();
    let (_com, auto) = started();
    let (one, two) = (TestEdit::start(LATIN, false), TestEdit::start(LATIN, false));
    let id = box_id(&one.element(&auto));
    assert_ne!(id, 0);
    assert_eq!(box_id(&one.element(&auto)), id);
    assert_ne!(box_id(&two.element(&auto)), id);
}

#[test]
fn the_words_before_the_caret_are_cut_to_the_last_n_in_latin_and_arabic() {
    let _one = one_client();
    let (_com, auto) = started();
    let (latin_box, arabic_box) = (
        TestEdit::start(LATIN, false),
        TestEdit::start(ARABIC, false),
    );
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
    let (secret_box, plain_box) = (TestEdit::start(LATIN, true), TestEdit::start(LATIN, false));
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
    let (plain, secret) = (TestEdit::start(LATIN, false), TestEdit::start(LATIN, true));
    seen(&shared, &plain.element(&auto), std::process::id());
    assert!(lock(&heard).is_empty(), "our own box is ignored");
    seen(&shared, &secret.element(&auto), 0);
    let told = FieldChange {
        kind: FieldKind::Password,
        box_id: box_id(&secret.element(&auto)),
    };
    assert_eq!(*lock(&heard), [told]);
    assert_eq!(run.kind(), FieldKind::Password);
}
