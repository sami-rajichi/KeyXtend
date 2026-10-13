//! How the shared graphics device is made (ADR-0021), apart from Qt: at once on the pinned adapter, or on Qt's default
//! when that adapter is gone, with one warning. A lost one, or one a window could not start drawing on, is made again
//! one try at a time on the adapter picked anew, never dropping the pin; the next try comes later, through Qt's loop,
//! and every try counts against `[ui] device_rebuilds`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

#[path = "hub/logs.rs"]
mod logs;

use kx_ui::UiSettings;
use kx_ui::device::{Cause, Next, Rebuilds, Remake, share};
use logs::warnings_of;
use std::cell::Cell;

/// What the fake maker makes.
const DEVICE: u8 = 7;
/// A device loss with every frame on the GUI thread.
const LOST: Cause = Cause::Lost { gui_only: true };

/// A maker that fails `fails` times, then makes `DEVICE`, and counts its tries.
fn maker(fails: u32, tries: &Cell<u32>) -> impl FnMut() -> Option<u8> + '_ {
    move || {
        tries.set(tries.get() + 1);
        (tries.get() > fails).then_some(DEVICE)
    }
}

#[test]
fn a_device_made_at_once_keeps_the_pin_and_warns_of_nothing() {
    let tries = Cell::new(0);
    let unpinned = Cell::new(false);
    let mut made = None;
    let warned = warnings_of(|| {
        made = share(maker(0, &tries), || {
            unpinned.set(true);
            true
        });
    });
    assert_eq!(made, Some(DEVICE));
    assert_eq!(tries.get(), 1);
    assert!(!unpinned.get() && warned.is_empty(), "{warned:?}");
}

#[test]
fn a_gone_pinned_adapter_is_dropped_and_the_device_made_on_the_default_with_one_warning() {
    let tries = Cell::new(0);
    let mut made = None;
    let warned = warnings_of(|| made = share(maker(1, &tries), || true));
    assert_eq!(made, Some(DEVICE));
    assert_eq!(tries.get(), 2, "one try on the pin, one on the default");
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(warned[0].contains("default"), "{warned:?}");
}

#[test]
fn no_device_on_the_pin_or_the_default_leaves_each_window_its_own_with_one_warning() {
    let tries = Cell::new(0);
    let mut made = None;
    let warned = warnings_of(|| made = share(maker(u32::MAX, &tries), || true));
    assert_eq!(made, None);
    assert_eq!(tries.get(), 2, "one try on the pin, one on the default");
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(warned[0].contains("its own"), "{warned:?}");
}

#[test]
fn no_device_without_a_pin_is_tried_once_with_one_warning() {
    let tries = Cell::new(0);
    let mut made = None;
    let warned = warnings_of(|| made = share(maker(u32::MAX, &tries), || false));
    assert_eq!((made, tries.get()), (None, 1));
    assert_eq!(warned.len(), 1, "{warned:?}");
}

/// What the fake remake did, in order.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Step {
    Repin,
    Make,
}

/// A remake whose first `fails` makes fail; it notes each step. It has nothing that waits, as `Remake` has none.
#[derive(Default)]
struct Fake {
    fails: usize,
    steps: Vec<Step>,
}

impl Fake {
    fn failing(fails: usize) -> Self {
        Self {
            fails,
            ..Self::default()
        }
    }
}

impl Remake for Fake {
    type Device = u8;

    fn repin(&mut self) {
        self.steps.push(Step::Repin);
    }

    fn make(&mut self) -> Option<u8> {
        self.steps.push(Step::Make);
        let tries = self.steps.iter().filter(|s| **s == Step::Make).count();
        (tries > self.fails).then_some(DEVICE)
    }
}

/// The most rebuilds the defaults allow.
fn most() -> u32 {
    UiSettings::from_defaults().unwrap().device_rebuilds
}

/// One try.
const TRY: [Step; 2] = [Step::Repin, Step::Make];

#[test]
fn a_lost_device_is_made_again_on_the_adapter_picked_anew_with_one_warning() {
    let mut fake = Fake::default();
    let mut next = Next::Own;
    let warned = warnings_of(|| next = Rebuilds::default().next(most(), LOST, &mut fake));
    assert_eq!(next, Next::Shared(DEVICE));
    assert_eq!(fake.steps, TRY);
    assert_eq!(warned.len(), 1, "{warned:?}");
}

#[test]
fn a_device_not_made_yet_is_tried_again_later_and_nothing_waits_here() {
    let (mut rebuilds, mut fake) = (Rebuilds::default(), Fake::failing(1));
    let mut first = Next::Own;
    let warned = warnings_of(|| first = rebuilds.next(most(), LOST, &mut fake));
    assert_eq!(
        first,
        Next::Later,
        "the caller tries again through Qt's loop"
    );
    assert_eq!(fake.steps, TRY, "one try, then back to the caller");
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(
        warned[0].contains("made yet"),
        "it names no loss: {warned:?}"
    );
    assert_eq!(rebuilds.next(most(), LOST, &mut fake), Next::Shared(DEVICE));
    assert_eq!(
        fake.steps,
        [TRY, TRY].concat(),
        "the pin is set anew, never dropped"
    );
}

#[test]
fn the_tries_count_against_the_setting_then_each_window_takes_its_own() {
    let (mut rebuilds, mut fake) = (Rebuilds::default(), Fake::failing(usize::MAX));
    let mut got = Vec::new();
    let warned = warnings_of(|| {
        got = (0..most())
            .map(|_| rebuilds.next(most(), LOST, &mut fake))
            .collect();
    });
    let later = usize::try_from(most()).unwrap() - 1;
    assert!(got[..later].iter().all(|n| *n == Next::Later), "{got:?}");
    assert_eq!(got[later], Next::Own);
    assert!(
        warned.last().unwrap().contains("each window makes its own"),
        "{warned:?}"
    );
    fake.steps.clear();
    assert_eq!(rebuilds.next(most(), LOST, &mut fake), Next::Own);
    assert!(fake.steps.is_empty(), "spent for the run");
}

#[test]
fn rebuilds_add_up_over_a_run() {
    let mut rebuilds = Rebuilds::default();
    let mut fake = Fake::default();
    assert_eq!(rebuilds.next(2, LOST, &mut fake), Next::Shared(DEVICE));
    assert_eq!(rebuilds.next(2, LOST, &mut fake), Next::Shared(DEVICE));
    fake.steps.clear();
    assert_eq!(rebuilds.next(2, LOST, &mut fake), Next::Own);
    assert!(fake.steps.is_empty(), "{:?}", fake.steps);
}

#[test]
fn with_no_rebuilds_the_first_loss_leaves_each_window_its_own() {
    let mut fake = Fake::default();
    assert_eq!(Rebuilds::default().next(0, LOST, &mut fake), Next::Own);
    assert!(fake.steps.is_empty());
}

#[test]
fn a_frame_off_the_gui_thread_leaves_each_window_its_own_at_once_with_one_warning() {
    let mut fake = Fake::default();
    let mut next = Next::Later;
    let off = Cause::Lost { gui_only: false };
    let warned = warnings_of(|| next = Rebuilds::default().next(most(), off, &mut fake));
    assert_eq!(next, Next::Own);
    assert!(fake.steps.is_empty(), "{:?}", fake.steps);
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(warned[0].contains("GUI thread"), "{warned:?}");
}

#[test]
fn a_window_that_cannot_draw_takes_the_rebuild_path_while_tries_remain() {
    let mut fake = Fake::default();
    let next = Rebuilds::default().next(most(), Cause::Failed, &mut fake);
    assert_eq!(next, Next::Shared(DEVICE));
}

#[test]
fn a_window_that_cannot_draw_once_the_tries_are_used_ends_the_app() {
    let (mut rebuilds, mut fake) = (Rebuilds::default(), Fake::failing(usize::MAX));
    assert_eq!(
        rebuilds.next(1, Cause::Failed, &mut fake),
        Next::End,
        "its last try failed"
    );
    fake.steps.clear();
    assert_eq!(
        rebuilds.next(1, Cause::Failed, &mut fake),
        Next::End,
        "none left"
    );
    assert!(fake.steps.is_empty());
}

#[cfg(feature = "qt")]
#[test]
fn no_device_is_made_before_qts_app_object_and_the_pin_stays_unwarned() {
    let pinned = 1;
    assert!(kx_ui::pin_adapter(pinned));
    let mut made = true;
    let warned = warnings_of(|| made = kx_ui::Device::make().is_some());
    assert!(!made && warned.is_empty(), "{warned:?}");
    assert!(kx_ui::qt_sees(kx_ui::ADAPTER_VAR), "the pin stays");
}
