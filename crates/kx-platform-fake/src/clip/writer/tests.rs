//! The fake clipboard writer, held to the port: every write recorded with its parts in order, and failures on cue.

use super::*;
use kx_module_api::Redacted;
use kx_module_api::ports::ImageCodec;
use std::path::PathBuf;
use std::thread;

/// How long a test waits for another thread; it never sleeps.
const WITHIN: Duration = Duration::from_secs(10);

/// Test values; the real ones come from the clipboard module's settings.
const WAIT: ClipWait = ClipWait {
    open_ms: 100,
    poll_ms: 10,
};

/// A text part.
fn text(t: &str) -> ClipPart {
    ClipPart::Text(Redacted::new(t.to_owned()))
}

/// A rich copy: HTML, its text, a picture and a file.
fn rich() -> Vec<ClipPart> {
    vec![
        ClipPart::Html(Redacted::new(b"<b>bonjour</b>".to_vec())),
        text("bonjour"),
        ClipPart::Image {
            codec: ImageCodec::Png,
            bytes: Redacted::new(vec![1, 2, 3]),
        },
        ClipPart::Files(Redacted::new(vec![PathBuf::from("C:/a.txt")])),
    ]
}

#[test]
fn each_write_is_recorded_with_its_parts_in_order() {
    let writer = FakeClipboardWriter::new();
    assert!(writer.writes().is_empty());
    assert_eq!(writer.last_wait(), None);
    assert_eq!(writer.write(&rich(), WAIT), Ok(()));
    assert_eq!(writer.write(&[text("مرحبا")], WAIT), Ok(()));
    assert_eq!(writer.writes(), [rich(), vec![text("مرحبا")]]);
    assert_eq!((writer.tries(), writer.last_wait()), (2, Some(WAIT)));
}

#[test]
fn queued_results_answer_writes_in_turn_then_writes_succeed() {
    let writer = FakeClipboardWriter::new();
    writer.queue(Ok(()));
    writer.queue(Err(ClipError::Busy));
    writer.queue(Err(ClipError::Refused));
    writer.queue(Err(ClipError::Io));
    let answers: Vec<_> = ["a", "b", "c", "d", "e"]
        .iter()
        .map(|t| writer.write(&[text(t)], WAIT))
        .collect();
    let errs = [ClipError::Busy, ClipError::Refused, ClipError::Io].map(Err);
    assert_eq!(
        answers,
        [vec![Ok(())], errs.to_vec(), vec![Ok(())]].concat()
    );
    assert_eq!(
        writer.writes(),
        [vec![text("a")], vec![text("e")]],
        "refused ones set nothing"
    );
    assert_eq!(writer.tries(), 5, "refused ones count");
}

#[test]
fn a_write_on_another_thread_wakes_a_waiting_test() {
    let writer = FakeClipboardWriter::new();
    assert!(!writer.wait_tries(1, Duration::ZERO), "no write yet");
    let port: Arc<dyn ClipboardWriter> = Arc::new(writer.clone());
    let module = thread::spawn(move || port.write(&[text("x")], WAIT));
    assert!(writer.wait_tries(1, WITHIN));
    assert_eq!(module.join().unwrap(), Ok(()));
    assert_eq!(writer.writes(), [vec![text("x")]]);
}

#[test]
fn debug_never_prints_the_parts() {
    let writer = FakeClipboardWriter::new();
    writer.write(&[text("secret words")], WAIT).unwrap();
    let shown = format!("{writer:?}");
    assert!(shown.contains("FakeClipboardWriter"));
    assert!(!shown.contains("secret"));
}
