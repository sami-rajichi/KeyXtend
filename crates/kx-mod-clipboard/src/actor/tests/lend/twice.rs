//! One app's double write (lend review, M1): a take that forgets the newest entry also forgets an unpinned entry of
//! that text alone copied within `take_wait_ms` of it; an older, pinned or unreadable copy of that text stays.

use kx_module_api::ports::{ClipAct, SealedError};
use kx_platform_fake::ShelfCall;

use super::super::super::Msg;
use super::super::logs::lines_of;
use super::super::rig::Rig;
use super::due::{STEP, texts};
use super::gone::{KEY, emptied, pasted, take};
use super::{holding, html};
use crate::fixture::text;

/// The voice key page, whose Paste takes the clipboard's text.
const ASKER: &str = "voice";

/// The key with HTML beside it, as an app's second write.
fn rich(rig: &mut Rig) {
    rig.copy(vec![text(KEY), html(KEY)], None);
}

#[test]
fn a_take_forgets_an_apps_text_alone_write_just_before_its_rich_one() {
    let mut rig = holding(vec![text(KEY)]);
    rich(&mut rig);
    take(&mut rig, ASKER);
    assert!(texts(&rig).is_empty(), "both writes forgotten");
    assert_eq!(pasted(&mut rig), emptied());
}

/// The rich write comes one step past `take_wait_ms` on the rig's own calendar.
#[test]
fn a_text_alone_copy_older_than_take_wait_ms_stays() {
    let mut rig = holding(vec![text(KEY)]);
    rig.fakes.calendar.advance(rig.settings.take_wait() + STEP);
    rich(&mut rig);
    take(&mut rig, ASKER);
    assert_eq!(texts(&rig), [KEY]);
}

#[test]
fn a_newest_entry_the_take_cannot_read_stays_and_its_warning_holds_no_text() {
    let mut rig = holding(vec![text(KEY)]);
    rig.fakes.shelf.fail(ShelfCall::Load, SealedError::Damaged);
    let lines = lines_of(|| take(&mut rig, ASKER));
    assert_eq!(texts(&rig), [KEY]);
    assert!(lines.iter().all(|l| !l.contains(KEY)), "{lines:?}");
    assert_eq!(lines.len(), 1, "{lines:?}");
}

/// The newest reads but the twin's blob will not: the newest goes, the twin stays, and Paste at once still never puts
/// its text back.
#[test]
fn a_twin_the_take_cannot_read_stays_and_its_warning_holds_no_text() {
    let mut rig = holding(vec![text(KEY)]);
    let twin = rig.actor.history.entries()[0].clone();
    rich(&mut rig);
    let (folder, name) = rig.actor.store.first_blob(&twin).unwrap();
    rig.fakes.shelf.plant(&folder, &name, Vec::new());
    let lines = lines_of(|| take(&mut rig, ASKER));
    let left: Vec<u64> = rig.actor.history.entries().iter().map(|e| e.id).collect();
    assert_eq!(left, [twin.id], "the newest went, the twin stays");
    assert!(lines.iter().all(|l| !l.contains(KEY)), "{lines:?}");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(pasted(&mut rig), emptied());
}

#[test]
fn a_pinned_text_alone_copy_stays() {
    let mut rig = holding(vec![text(KEY)]);
    rig.open();
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Pin));
    rig.send(Msg::Act(ClipAct::Close));
    rich(&mut rig);
    take(&mut rig, ASKER);
    assert_eq!(texts(&rig), [KEY]);
}
