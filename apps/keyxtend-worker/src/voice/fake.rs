//! Test parts for the voice job: a source the test drives by hand, a detector that hears loud frames, and an engine
//! whose writer records each piece and tells a note for it, so no test opens a microphone or loads a model.

use super::Tell;
use super::cut::{Detector, Detectors};
use super::engine::{Engine, Writer};
use super::job::Parts;
use super::mic::{Format, Opened, Source};
use super::wipe::Buf;
use crate::voice::Feed;
use kx_ipc::{Model, SOUND_HZ, VoiceFault, VoiceNote};
use std::cell::RefCell;
use std::rc::Rc;
use zeroize::Zeroizing;

/// A sample the detector hears as speech; quieter than half of it is silence.
pub const LOUD: i16 = 10_000;
/// The frame the job tests' detector judges, 10 ms.
pub const FRAME: usize = SOUND_HZ / 100;
/// The text the fake writer tells for every piece.
pub const WORDS: &str = "words";

/// Hears a frame as speech when any sample is loud.
pub struct Loud(pub usize);

impl Detector for Loud {
    fn frame(&self) -> usize {
        self.0
    }

    fn speech(&mut self, frame: &[i16]) -> bool {
        frame
            .iter()
            .any(|s| s.unsigned_abs() > LOUD.unsigned_abs() / 2)
    }
}

/// Opens a `Loud` detector, or none.
pub struct FakeDetectors(pub bool);

impl Detectors for FakeDetectors {
    fn open(&mut self, _: &str) -> Result<Option<Box<dyn Detector>>, VoiceFault> {
        Ok(self.0.then(|| Box::new(Loud(FRAME)) as Box<dyn Detector>))
    }
}

/// What the test sees of its source.
pub struct Mic {
    /// The format it opens with.
    pub format: Format,
    /// The fault it opens with instead, if any.
    pub refuse: Option<VoiceFault>,
    /// The feed of its last opening.
    pub feed: Option<Feed>,
    /// True while it is open.
    pub open: bool,
    /// How many times it opened.
    pub opens: usize,
}

/// A source the test drives through its shared `Mic`.
pub struct FakeSource(pub Rc<RefCell<Mic>>);

/// Closes the fake source when dropped, as dropping a real stream does.
struct Guard(Rc<RefCell<Mic>>);

impl Drop for Guard {
    fn drop(&mut self) {
        self.0.borrow_mut().open = false;
    }
}

impl Source for FakeSource {
    fn open(&mut self, feed: Feed) -> Result<Opened, VoiceFault> {
        let mut mic = self.0.borrow_mut();
        if let Some(fault) = mic.refuse {
            return Err(fault);
        }
        (mic.feed, mic.open, mic.opens) = (Some(feed), true, mic.opens + 1);
        let live = Box::new(Guard(Rc::clone(&self.0)));
        Ok(Opened {
            format: mic.format,
            live,
        })
    }
}

/// What the fake engine's writers were given.
#[derive(Debug, PartialEq, Eq)]
pub enum Got {
    /// A piece's samples.
    Piece(Vec<i16>),
    /// The end, with the sound heard.
    End(u32),
}

/// An engine whose writers log what they get in `log` and tell a piece note for each piece and a done note at the end.
pub struct FakeEngine(pub Rc<RefCell<Vec<Got>>>);

struct FakeWriter {
    id: u32,
    told: u16,
    tell: Tell,
    log: Rc<RefCell<Vec<Got>>>,
}

impl Engine for FakeEngine {
    fn load(&mut self, id: u32, _: &Model<'_>, tell: Tell) -> Result<Box<dyn Writer>, VoiceFault> {
        let log = Rc::clone(&self.0);
        Ok(Box::new(FakeWriter {
            id,
            told: 0,
            tell,
            log,
        }))
    }
}

impl Writer for FakeWriter {
    fn piece(&mut self, sound: Buf<i16>) {
        self.log.borrow_mut().push(Got::Piece(sound.to_vec()));
        let text = Zeroizing::new(WORDS.to_owned());
        let (id, index) = (self.id, self.told);
        self.tell.note(VoiceNote::Piece { id, index, text });
        self.told += 1;
    }

    fn end(&mut self, heard_ms: u32) {
        self.log.borrow_mut().push(Got::End(heard_ms));
        self.tell.note(done(self.id, self.told, heard_ms));
    }
}

/// The done note a run of `pieces` and `sound_ms` ends with, as the fake writer and the listen-only job tell it.
pub fn done(id: u32, pieces: u16, sound_ms: u32) -> VoiceNote {
    VoiceNote::Done {
        id,
        pieces,
        sound_ms,
        wait_ms: 0,
        load_ms: 0,
        memory_mb: 0,
        card: false,
    }
}

/// Test parts over `mic`: a detector when `detector`, and the fake engine logging to `log`.
pub fn parts(mic: &Rc<RefCell<Mic>>, detector: bool, log: &Rc<RefCell<Vec<Got>>>) -> Parts {
    Parts {
        source: Box::new(FakeSource(Rc::clone(mic))),
        detectors: Box::new(FakeDetectors(detector)),
        engine: Box::new(FakeEngine(Rc::clone(log))),
    }
}

/// A source at `rate` with `channels`, closed.
pub fn mic(rate: u32, channels: u16) -> Rc<RefCell<Mic>> {
    Rc::new(RefCell::new(Mic {
        format: Format { rate, channels },
        refuse: None,
        feed: None,
        open: false,
        opens: 0,
    }))
}
