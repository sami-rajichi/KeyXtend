//! Any run of device losses and windows that cannot draw (ADR-0021): the tries never pass `[ui] device_rebuilds`, each
//! pins the adapter anew before it makes a device, and once they are used each window takes its own device, or the app
//! ends when a window cannot draw. The shell's loop, which asks again at once when nothing can time a try, always ends.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

use kx_ui::UiSettings;
use kx_ui::device::{Cause, Next, Rebuilds, Remake};
use proptest::collection::vec;
use proptest::prelude::*;

/// What the fake maker makes.
const DEVICE: u8 = 7;
/// A device loss with every frame on the GUI thread.
const LOST: Cause = Cause::Lost { gui_only: true };

/// A remake that makes a device only while `can` holds; it counts its pins and makes, and notes a make with no new pin.
#[derive(Default)]
struct Fake {
    can: bool,
    pins: u32,
    makes: u32,
    unpinned: bool,
}

impl Remake for Fake {
    type Device = u8;

    fn repin(&mut self) {
        self.pins += 1;
    }

    fn make(&mut self) -> Option<u8> {
        self.unpinned |= self.pins <= self.makes;
        self.makes += 1;
        self.can.then_some(DEVICE)
    }
}

/// The most rebuilds the defaults allow.
fn most() -> u32 {
    UiSettings::from_defaults().unwrap().device_rebuilds
}

/// Why the windows are made again: a loss on or off the GUI thread, or a window that cannot draw.
fn cause() -> impl Strategy<Value = Cause> {
    prop_oneof![
        any::<bool>().prop_map(|gui_only| Cause::Lost { gui_only }),
        Just(Cause::Failed),
    ]
}

/// Checks one call that found `fake` with `before` makes: one try, pinned anew, exactly while tries remain and the
/// frame drew on the GUI thread, and what follows as the tries left allow.
fn check(
    fake: &Fake,
    before: u32,
    most: u32,
    cause: Cause,
    next: Next<u8>,
) -> Result<(), TestCaseError> {
    let tried = fake.makes > before;
    prop_assert!(fake.makes <= most, "{} tries of {}", fake.makes, most);
    prop_assert!(fake.makes - before <= 1, "one try a call");
    prop_assert!(
        fake.pins == fake.makes && !fake.unpinned,
        "each try pins anew, never dropping the pin"
    );
    let left = cause != Cause::OFF_THREAD && before < most;
    prop_assert_eq!(
        tried,
        left,
        "a try only while tries remain, and never off the GUI thread"
    );
    let (made, spent) = (tried && fake.can, fake.makes == most);
    prop_assert_eq!(
        made,
        matches!(next, Next::Shared(_)),
        "a device made is shared"
    );
    match next {
        Next::Shared(device) => prop_assert_eq!(device, DEVICE),
        Next::Later => prop_assert!(tried && !spent, "later only while tries remain"),
        Next::Own => prop_assert!(cause == Cause::OFF_THREAD || (cause == LOST && spent)),
        Next::End => prop_assert!(
            cause == Cause::Failed && spent,
            "only a window that cannot draw"
        ),
    }
    Ok(())
}

proptest! {
    #[test]
    fn rebuilds_never_pass_the_setting_then_leave_each_window_its_own_or_end(
        most in 0..=most(),
        steps in vec((cause(), any::<bool>()), 0..40),
    ) {
        let (mut rebuilds, mut fake) = (Rebuilds::default(), Fake::default());
        for (cause, can) in steps {
            fake.can = can;
            let before = fake.makes;
            let next = rebuilds.next(most, cause, &mut fake);
            check(&fake, before, most, cause, next)?;
        }
        fake.can = false;
        let mut asks = 0;
        while rebuilds.next(most, LOST, &mut fake) == Next::Later {
            asks += 1;
            prop_assert!(asks < most, "the shell's loop ends once the tries are used");
        }
    }
}
