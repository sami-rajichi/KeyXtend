//! Focus in one of our own windows: looks keep the last box and say it is ours, and `still` and reads refuse it.

use super::*;

#[test]
fn our_own_focus_keeps_the_last_box_refuses_still_and_reads_and_a_new_focus_ends_it() {
    let fake = FakeSelectionWatch::new();
    let (mut run, _) = watched(&fake);
    fake.set_text(TEXT);
    fake.focus(selected());
    let theirs = run.look();
    assert!(!run.ours(), "another app's box");
    fake.focus_ours();
    assert_eq!(run.look(), theirs, "the last box");
    assert!(run.ours());
    assert!(!run.still(fake.spot().box_id));
    let reads = fake.calls().reads;
    assert_eq!(read(&fake, run.as_mut(), 9), Err(SelError::Closed));
    assert_eq!(fake.calls().reads, reads, "a refused read is not counted");
    fake.focus(selected());
    assert!(run.look().is_ok() && !run.ours(), "focus left our window");
}
