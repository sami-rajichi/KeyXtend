//! The fakes behind the actor's ports, at the rig's start time and day.

use kx_module_api::Mono;
use kx_platform_fake::{
    FakeCalendar, FakeClipboard, FakeClipboardWriter, FakeClock, FakePictures, FakeShelf,
};

use crate::fixture::{PARIS, date};

/// When each test starts.
const T0: Mono = Mono::from_us(1_000_000);

/// The fakes behind the actor's ports.
#[derive(Clone)]
pub(super) struct Fakes {
    pub clip: FakeClipboard,
    pub writer: FakeClipboardWriter,
    pub shelf: FakeShelf,
    pub pics: FakePictures,
    pub clock: FakeClock,
    pub calendar: FakeCalendar,
}

impl Fakes {
    /// Fresh fakes at `T0`, on 31 October 2026 at 16:32 in Paris.
    pub(super) fn new() -> Self {
        Self {
            clip: FakeClipboard::new(),
            writer: FakeClipboardWriter::new(),
            shelf: FakeShelf::new(),
            pics: FakePictures::new(),
            clock: FakeClock::new(T0),
            calendar: FakeCalendar::new(date(2026, 10, 31), 16, 32, PARIS),
        }
    }
}
