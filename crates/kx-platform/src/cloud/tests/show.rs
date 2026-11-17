//! `Debug` of what the host holds and tells never shows the key or a text: the asks, the frames built from them, a
//! board's step, each note and each fault.

use super::*;
use crate::cloud::board::Step;

#[test]
fn debug_never_shows_the_key_or_a_text() {
    let note = piece(0, SAID);
    let step = Step {
        notes: vec![note.clone(), CloudNote::Done],
        cancel: true,
    };
    let test = key_test();
    let call = frame::call(test.shape, &test.url, test.header, test.key, test.wait_s);
    let faults = [
        CloudFault::Range(SHAPE),
        CloudFault::Key,
        CloudFault::Allowance { renews_in: None },
        CloudFault::Strange,
        CloudFault::Gone,
    ];
    let mut shown = vec![
        format!("{note:?}"),
        format!("{step:?}"),
        format!("{:?}", ask(&[PER_S])),
        format!("{:?}", key_test()),
        format!("{call:?}"),
    ];
    shown.extend(faults.iter().map(|f| format!("{f:?}")));
    for s in shown {
        assert!(!s.contains(KEY) && !s.contains(SAID), "{s}");
    }
}
