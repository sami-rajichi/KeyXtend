//! Caps Lock in the typing mode is the one the UI's thread last read: Windows reads it per thread, so a mode built on
//! the mouse hook's thread, a `LatchesAsk` sender's or the prediction module's never reads it there.

use super::Desk;
use crate::held::A;
use crate::host::{Fakes, GOLDEN};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    FaceAction, KeyChars, LangFacts, LatchesAsk, LatchesUsed, LayoutId, LayoutSource, Scan,
    SwitchError, TypeWord, TypingMode,
};
use kx_module_api::{Bus, Event};
use kx_platform_fake::FakeLayoutSource;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::thread::{self, ThreadId};

/// The fake source, read the other way round off the UI's thread, as a per-thread read could be.
struct UiCaps {
    fake: Arc<FakeLayoutSource>,
    ui: ThreadId,
    /// Caps Lock reads off the UI's thread.
    off: AtomicUsize,
}

impl LayoutSource for UiCaps {
    fn installed(&self) -> Vec<LayoutId> {
        self.fake.installed()
    }

    fn foreground(&self) -> Option<LayoutId> {
        self.fake.foreground()
    }

    fn keys(&self, layout: LayoutId, scans: &[Scan]) -> Vec<KeyChars> {
        self.fake.keys(layout, scans)
    }

    fn facts(&self, layout: LayoutId) -> LangFacts {
        self.fake.facts(layout)
    }

    fn caps_on(&self) -> bool {
        let on = self.fake.caps_on();
        if thread::current().id() == self.ui {
            return on;
        }
        self.off.fetch_add(1, SeqCst);
        !on
    }

    fn types_any(&self, layout: LayoutId, key: Scan, held: &[Scan]) -> bool {
        self.fake.types_any(layout, key, held)
    }

    fn request(&self, layout: LayoutId) -> Result<(), SwitchError> {
        self.fake.request(layout)
    }
}

/// Sends `event` from a thread of its own, as another module does.
fn from_elsewhere<E: Event>(bus: &Bus, event: &E) {
    thread::scope(|s| {
        s.spawn(|| bus.publish(event).unwrap());
    });
}

#[test]
fn a_mode_built_off_the_ui_thread_keeps_the_caps_lock_the_ui_saw() {
    let mut fakes = Fakes::golden(&GOLDEN);
    let caps = Arc::new(UiCaps {
        fake: Arc::clone(&fakes.source),
        ui: thread::current().id(),
        off: AtomicUsize::new(0),
    });
    fakes.served = Some(caps.clone());
    let d = Desk::on(fakes, None);
    d.rig.fakes.source.set_caps(true);
    d.rig.act(FaceAction::Relabel).unwrap();
    d.rig.tap("shift").unwrap();
    let lit = TypingMode {
        caps: true,
        ..TypingMode::default()
    };
    let shifted = TypingMode { shift: true, ..lit };
    assert_eq!(d.modes()[1..], [lit, shifted], "read on the UI's thread");
    from_elsewhere(&d.bus, &LatchesUsed);
    assert_eq!(d.modes(), [lit], "the hook's thread keeps it lit");
    from_elsewhere(&d.bus, &LatchesAsk);
    let word = TypeWord {
        round: 0,
        erase: 1,
        text: Redacted::new("ab".into()),
    };
    from_elsewhere(&d.bus, &word);
    d.rig.tap(A).unwrap();
    assert!(d.modes().is_empty(), "nothing changed");
    assert_eq!(caps.off.load(SeqCst), 0, "never read off the UI's thread");
}
