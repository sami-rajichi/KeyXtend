//! The last note of a run is handed back, not told, so the job knows the run is over before the host reads it: the end,
//! the fault that ended it, nothing after Cancel, or `Failed` when the video could not be deleted.

use super::*;

/// Records `plan` over `script` with `orders`; the notes told, and the note handed back.
fn told(
    script: Script,
    plan: &Plan,
    orders: Vec<(Duration, Order)>,
) -> (Vec<RecordNote>, Option<RecordNote>) {
    let (studio, rx) = FakeStudio::new(script, orders);
    let mut notes = Vec::new();
    let last = record(&studio, plan, &rx, &mut |n| notes.push(n));
    (notes, last)
}

#[test]
fn the_end_is_handed_back_after_the_times() {
    let temp = Temp::new();
    let (notes, last) = told(script(), &plan(temp.path()), vec![(secs(1.5), Order::Stop)]);
    assert_eq!(times(&notes), [0, 1000]);
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert_eq!(last, Some(end(1500, RecordEnd::Stopped)));
}

#[test]
fn a_fault_that_ends_a_run_is_handed_back_and_cancel_hands_back_nothing() {
    let temp = Temp::new();
    let mut s = script();
    s.ready = Err(RecordFault::NoVideo);
    let (notes, last) = told(s, &plan(temp.path()), Vec::new());
    assert_eq!(
        (notes, last),
        (Vec::new(), Some(fault(RecordFault::NoVideo)))
    );
    let mut s = script();
    s.sink_fails_at = Some(3);
    let (notes, last) = told(s, &plan(temp.path()), Vec::new());
    assert_eq!(
        (times(&notes), last),
        (vec![0], Some(fault(RecordFault::Failed)))
    );
    let cancel = vec![(secs(0.5), Order::Cancel)];
    let (notes, last) = told(script(), &plan(temp.path()), cancel);
    assert_eq!((times(&notes), last), (vec![0], None));
}

#[test]
fn a_video_that_cannot_be_deleted_ends_in_failed_for_the_host_to_delete() {
    let temp = Temp::new();
    let p = plan(temp.path());
    let mut s = script();
    s.held = true;
    let (notes, last) = told(s, &p, vec![(secs(0.5), Order::Cancel)]);
    let failed = Some(fault(RecordFault::Failed));
    assert_eq!((times(&notes), last), (vec![0], failed));
    assert!(p.file.exists());
}

#[test]
fn scrap_deletes_a_file_counts_a_missing_one_gone_and_says_when_it_cannot() {
    let temp = Temp::new();
    let file = temp.path().join("a.mp4");
    fs::write(&file, b"video").unwrap();
    assert!(scrap(&file) && !file.exists());
    assert!(scrap(&file));
    let held = temp.path().join("held.mp4");
    fs::create_dir(&held).unwrap();
    assert!(!scrap(&held) && held.exists());
}
