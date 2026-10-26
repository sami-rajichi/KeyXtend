//! A run told a stale `Text` kind: a box that is now a password box is never read, and the read reports its kind.

use super::super::{TEXT_CALLS, before_cursor, ends_with};
use super::{LATIN, chars, closed, text};
use crate::field::testedit::{CONFIG, TestEdit, element, started};
use crate::field::win::WinProbe;
use crate::scroll::testwin::{WAIT, one_client};
use kx_module_api::ports::{FieldChange, FieldError, FieldKind, FieldRead};
use kx_platform::field::{Job, Probe, Reader, Shared, serve, spawn};
use std::cell::Cell;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;
use windows::Win32::UI::Accessibility::IUIAutomationElement;
use windows::Win32::UI::WindowsAndMessaging::ES_PASSWORD;

/// A reader whose focus is one box of another app, read as the watch reads.
struct Pinned(IUIAutomationElement);

impl Probe for Pinned {
    type El = IUIAutomationElement;

    fn ours(&self, _: &IUIAutomationElement) -> bool {
        false
    }

    fn change(&self, el: &IUIAutomationElement) -> FieldChange {
        WinProbe::new().change(el)
    }
}

impl Reader for Pinned {
    fn focused(&self) -> Result<IUIAutomationElement, FieldError> {
        Ok(self.0.clone())
    }

    fn before_cursor(
        &self,
        el: &IUIAutomationElement,
        max_words: u8,
        by: Instant,
    ) -> Result<FieldRead, FieldError> {
        before_cursor(el, max_words, by, chars())
    }

    fn ends_with(
        &self,
        el: &IUIAutomationElement,
        text: &str,
        by: Instant,
    ) -> Result<Option<bool>, FieldError> {
        ends_with(el, text, by, chars())
    }
}

/// A probe that tells a scripted box as another app's.
struct Told;

impl Probe for Told {
    type El = FieldChange;

    fn ours(&self, _: &FieldChange) -> bool {
        false
    }

    fn change(&self, change: &FieldChange) -> FieldChange {
        *change
    }
}

/// What a run told a stale `Text` reads from the box with raw handle `raw`, and how many text calls its thread made;
/// its thread and client are gone on return.
fn stale_read(raw: isize) -> (Result<FieldRead, FieldError>, usize) {
    let (give, got) = mpsc::channel();
    let (count, counted) = mpsc::channel();
    let (ended, done) = mpsc::channel();
    let body = move |inbox: &Receiver<Job>, shared: &Arc<Shared>, _| {
        let _ = give.send(Arc::clone(shared));
        let (com, auto) = started();
        let pinned = Pinned(element(&auto, raw));
        for job in inbox {
            serve(&pinned, job);
            let _ = count.send(TEXT_CALLS.with(Cell::get));
        }
        drop((pinned, auto, com));
        let _ = ended.send(());
    };
    let mut run = spawn(CONFIG, Box::new(|_: FieldChange| {}), body).unwrap();
    let shared: Arc<Shared> = got.recv_timeout(WAIT).unwrap();
    let text = FieldChange {
        kind: FieldKind::Text,
        box_id: 1,
    };
    shared.moved(&Told, &text);
    let read = run.before_cursor(2);
    let calls = counted.recv_timeout(WAIT).unwrap();
    drop(run);
    done.recv_timeout(WAIT).unwrap();
    (read, calls)
}

#[test]
fn a_stale_text_kind_never_reads_a_box_that_is_now_a_password_box_and_reports_its_kind() {
    let _one = one_client();
    let (secret, plain) = (
        TestEdit::start(LATIN, ES_PASSWORD),
        TestEdit::start(LATIN, 0),
    );
    assert_eq!(
        stale_read(secret.raw()),
        (Ok(closed(FieldKind::Password)), 0)
    );
    let (read, calls) = stale_read(plain.raw());
    assert_eq!(read.as_ref().map(|r| r.kind), Ok(FieldKind::Text));
    assert_eq!(text(read), Ok(Some("gamma delta".to_owned())));
    assert!(calls > 0, "the plain box is read");
}
