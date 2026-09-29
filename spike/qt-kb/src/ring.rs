//! The `RingData` QObject: gate G12's ring test, a loop of holds at the pointer whose frame gaps are counted.

use std::pin::Pin;
use std::time::Instant;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use kx_test_support::ringstats::{self, Frames};
use spike_core::holdcfg::RingTest;

use crate::bridge::{config, ms};

/// The cxx-qt bridge that makes `RingData` a QML type.
#[cxx_qt::bridge]
pub mod qobject {
    // SAFETY: the header and type are cxx-qt-lib's own QString binding, so both sides match.
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, title, READ, CONSTANT)]
        #[qproperty(QString, label, READ, CONSTANT)]
        #[qproperty(i32, after_ms, READ, CONSTANT)]
        #[qproperty(i32, wave_ms, READ, CONSTANT)]
        #[qproperty(i32, gap_ms, READ, CONSTANT)]
        type RingData = super::RingRust;

        /// Starts a new frame record.
        #[qinvokable]
        fn start(self: Pin<&mut Self>);

        /// Counts one frame, timed from the one before by the clock.
        #[qinvokable]
        fn frame(self: Pin<&mut Self>);

        /// Writes the record's stats; returns an error note or empty.
        #[qinvokable]
        fn stop(&self) -> QString;
    }
}

/// Rust side of `RingData`.
pub struct RingRust {
    title: QString,
    label: QString,
    after_ms: i32,
    wave_ms: i32,
    gap_ms: i32,
    test: RingTest,
    rec: Frames,
}

impl Default for RingRust {
    fn default() -> Self {
        let cfg = config();
        let (hold, test) = (&cfg.hold, cfg.ring.clone());
        Self {
            title: QString::from(&test.title),
            label: QString::from(&test.label),
            after_ms: ms(hold.ring_after_ms),
            wave_ms: ms(hold.wave_ms()),
            gap_ms: ms(test.gap_ms),
            rec: Frames::new(test.max_frames),
            test,
        }
    }
}

impl qobject::RingData {
    fn start(mut self: Pin<&mut Self>) {
        let cap = self.rust().test.max_frames;
        self.as_mut().rust_mut().rec = Frames::new(cap);
    }

    fn frame(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().rec.tick(Instant::now());
    }

    fn stop(&self) -> QString {
        let r = self.rust();
        let done = r
            .test
            .stats_path()
            .and_then(|file| ringstats::save(&file, &r.rec.report()).map_err(|e| e.to_string()));
        QString::from(&done.err().unwrap_or_default())
    }
}
