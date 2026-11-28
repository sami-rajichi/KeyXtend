//! Re-timed copies: 2× ends with half the length, a copy that fails leaves nothing and says why, and a note that fails
//! its own checks goes as a failure.

use super::*;
use crate::stream::{Note, Pipe};
use kx_ipc::MAX_VIDEO_MS;

/// A copy `id` of `from` at `times` into `to`.
pub(super) fn ask(id: u32, from: &Path, times: u8, to: &Path) -> RecordAsk<'static> {
    let path = |p: &Path| Cow::Owned(p.to_string_lossy().into_owned());
    RecordAsk::Speed {
        id,
        from: path(from),
        speed: times,
        to: path(to),
    }
}

#[test]
fn two_times_ends_with_half_the_length_and_keeps_the_copy() {
    let dir = Dir::new();
    let (from, to) = (dir.file("snip 1.mp4"), dir.file("snip 1 2x.mp4"));
    fs::write(&from, b"video").unwrap();
    let (studio, _) = FakeStudio::new(script(), Vec::new());
    let mut host = Host::start(&studio);
    host.send(&ask(ID, &from, 2, &to));
    assert_eq!(host.next(), end(500, RecordEnd::Stopped));
    assert_eq!(fs::read(&to).unwrap(), ENDED);
    assert_eq!(fs::read(&from).unwrap(), b"video");
}

#[test]
fn a_copy_that_fails_leaves_nothing_and_says_why() {
    let dir = Dir::new();
    let (from, to) = (dir.file("snip 1.mp4"), dir.file("snip 1 4x.mp4"));
    fs::write(&from, b"video").unwrap();
    let mut s = script();
    s.tape.frames = 0;
    let (studio, _) = FakeStudio::new(s, Vec::new());
    let mut host = Host::start(&studio);
    host.send(&ask(ID, &from, 4, &to));
    let failed = RecordNote::Fault {
        id: ID,
        fault: RecordFault::Failed,
    };
    assert_eq!(host.next(), failed);
    assert!(!to.exists());
    studio.world().script.ready = Err(RecordFault::NoVideo);
    host.send(&ask(ID + 1, &from, 4, &to));
    let fault = RecordFault::NoVideo;
    assert_eq!(host.next(), RecordNote::Fault { id: ID + 1, fault });
    assert!(!to.exists());
}

#[test]
fn a_note_past_its_limit_goes_as_a_failure() {
    let mut out = Vec::new();
    let late = RecordNote::Time {
        id: ID,
        ms: MAX_VIDEO_MS + 1,
    };
    Pipe(&mut out).send(&late).unwrap();
    let fault = RecordFault::Failed;
    assert_eq!(
        RecordNote::read(&mut &out[..]),
        Ok(RecordNote::Fault { id: ID, fault })
    );
    assert_eq!(late.instead(), RecordNote::Fault { id: ID, fault });
}
