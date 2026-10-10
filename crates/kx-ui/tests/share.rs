//! How the shared graphics device is made (ADR-0021), apart from Qt: at once on the pinned adapter, or on Qt's default
//! when that adapter is gone, with one warning; a lost one is made again on the adapter picked anew, never dropping
//! the pin, each try counted against `[ui] device_rebuilds`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

#[path = "hub/logs.rs"]
mod logs;

use kx_ui::UiSettings;
use kx_ui::device::{Rebuilds, Remake, share};
use logs::warnings_of;
use std::cell::Cell;

/// What the fake maker makes.
const DEVICE: u8 = 7;

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
    Pause,
}

/// A remake whose first `fails` makes fail; it notes each step.
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

    fn pause(&mut self) {
        self.steps.push(Step::Pause);
    }
}

/// The most rebuilds the defaults allow.
fn most() -> u32 {
    UiSettings::from_defaults().unwrap().device_rebuilds
}

#[test]
fn a_lost_device_is_made_again_on_the_adapter_picked_anew_with_one_warning() {
    let mut fake = Fake::default();
    let mut made = None;
    let warned = warnings_of(|| made = Rebuilds::default().remake(most(), true, &mut fake));
    assert_eq!(made, Some(DEVICE));
    assert_eq!(fake.steps, [Step::Repin, Step::Make]);
    assert_eq!(warned.len(), 1, "{warned:?}");
}

#[test]
fn a_device_not_made_yet_is_tried_again_after_a_pause_and_the_pin_stays() {
    let mut fake = Fake::failing(1);
    let mut made = None;
    let warned = warnings_of(|| made = Rebuilds::default().remake(most(), true, &mut fake));
    assert_eq!(made, Some(DEVICE), "no try drops the pin for Qt's default");
    let tries = [Step::Repin, Step::Make];
    assert_eq!(
        fake.steps,
        [&tries[..], &[Step::Pause], &tries[..]].concat()
    );
    assert_eq!(
        warned.len(),
        2,
        "the failed try and the rebuild: {warned:?}"
    );
}

#[test]
fn the_tries_count_against_the_setting_then_each_window_takes_its_own() {
    let mut rebuilds = Rebuilds::default();
    let mut fake = Fake::failing(usize::MAX);
    let mut made = Some(DEVICE);
    let warned = warnings_of(|| made = rebuilds.remake(most(), true, &mut fake));
    assert_eq!(made, None);
    let tries = fake.steps.iter().filter(|s| **s == Step::Make).count();
    assert_eq!(u32::try_from(tries).unwrap(), most());
    assert!(warned.last().unwrap().contains("too often"), "{warned:?}");
    fake.steps.clear();
    assert_eq!(rebuilds.remake(most(), true, &mut fake), None);
    assert!(fake.steps.is_empty(), "spent for the run");
}

#[test]
fn rebuilds_add_up_over_a_run() {
    let mut rebuilds = Rebuilds::default();
    let mut fake = Fake::default();
    assert_eq!(rebuilds.remake(2, true, &mut fake), Some(DEVICE));
    assert_eq!(rebuilds.remake(2, true, &mut fake), Some(DEVICE));
    fake.steps.clear();
    assert_eq!(rebuilds.remake(2, true, &mut fake), None);
    assert!(fake.steps.is_empty(), "{:?}", fake.steps);
}

#[test]
fn with_no_rebuilds_the_first_loss_leaves_each_window_its_own() {
    let mut fake = Fake::default();
    assert_eq!(Rebuilds::default().remake(0, true, &mut fake), None);
    assert!(fake.steps.is_empty());
}

#[test]
fn a_frame_off_the_gui_thread_leaves_each_window_its_own_at_once_with_one_warning() {
    let mut fake = Fake::default();
    let mut made = Some(DEVICE);
    let warned = warnings_of(|| made = Rebuilds::default().remake(most(), false, &mut fake));
    assert_eq!(made, None);
    assert!(fake.steps.is_empty(), "{:?}", fake.steps);
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(warned[0].contains("GUI thread"), "{warned:?}");
}

#[cfg(feature = "qt")]
#[test]
fn no_device_is_made_before_qts_app_object() {
    assert!(kx_ui::Device::make().is_none());
}
