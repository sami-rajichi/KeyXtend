//! The fake recorder's log of recordings and what the test set, and the one path every note takes: a recording's
//! callback is held while its note is marked and told, so a cancel or a drop waits out a note being told.

use super::{Heard, RecLog, RecordCall};
use crate::lock;
use crate::snip::folder::{FakeSnipFolder, Stored};
use kx_module_api::Mono;
use kx_module_api::ports::{
    FileKind, FolderPath, RecordAsk, RecordError, RecordFacts, RecordNote, SnipFile, Source,
    SpeedAsk,
};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

/// The callback a recording tells its notes to.
pub(super) type Tell = Box<dyn Fn(RecordNote) + Send>;
/// A recording's callback, shared so it is called with the log unlocked.
pub(super) type Hearer = Arc<Mutex<Tell>>;

/// Whole ms from `from` to `to`, at most `u32::MAX`.
pub(super) fn ms(from: Mono, to: Mono) -> u32 {
    u32::try_from(to.saturating_since(from).as_millis()).unwrap_or(u32::MAX)
}

/// One recording: its log, when it started, what it hears now, and its callback until it ends.
pub(super) struct Run {
    pub(super) log: RecLog,
    pub(super) at: Mono,
    pub(super) now: Heard,
    hearer: Option<Hearer>,
}

impl Run {
    /// A recording of `ask` into `file` from `at`, hearing what the ask switched on.
    pub(super) fn new(ask: RecordAsk, file: SnipFile, at: Mono, tell: Tell) -> Self {
        let (mic, sound) = (ask.mic.is_some(), ask.sound);
        let now = Heard { ms: 0, mic, sound };
        let log = RecLog {
            ask,
            file,
            heard: vec![now],
            told: Vec::new(),
            stops: 0,
            cancels: 0,
            live: true,
            dropped: false,
        };
        let hearer = Some(Arc::new(Mutex::new(tell)));
        Self {
            log,
            at,
            now,
            hearer,
        }
    }

    /// Ends it, so it records and hears no more, and gives its callback, to wait out a note being told.
    pub(super) fn end(&mut self) -> Option<Hearer> {
        self.log.live = false;
        self.hearer.take()
    }
}

/// What the recordings did, and what the test set for them.
pub(super) struct Log {
    pub(super) facts: RecordFacts,
    /// Every start asked, refused ones too.
    pub(super) starts: Vec<RecordAsk>,
    pub(super) runs: Vec<Run>,
    pub(super) fails: HashMap<RecordCall, RecordError>,
    pub(super) speeds: Vec<SpeedAsk>,
    /// Videos made so far, which names the next.
    made: u32,
}

impl Log {
    /// No recording yet, with `facts`.
    pub(super) fn new(facts: RecordFacts) -> Self {
        Self {
            facts,
            starts: Vec::new(),
            runs: Vec::new(),
            fails: HashMap::new(),
            speeds: Vec::new(),
            made: 0,
        }
    }

    /// Each recording's log, oldest first.
    pub(super) fn logs(&self) -> Vec<RecLog> {
        self.runs.iter().map(|r| r.log.clone()).collect()
    }

    /// The newest live recording.
    pub(super) fn newest(&self) -> Option<usize> {
        self.runs.iter().rposition(|r| r.log.live)
    }

    /// Refuses a call on recording `id` once it ended, as its job no longer answers.
    pub(super) fn gone(&self, id: usize) -> Result<(), RecordError> {
        let live = self.runs[id].log.live;
        live.then_some(()).ok_or(RecordError::Gone)
    }

    /// The failure set for `call`, taken once.
    pub(super) fn failed(&mut self, call: RecordCall) -> Result<(), RecordError> {
        self.fails.remove(&call).map_or(Ok(()), Err)
    }

    /// A new video in `unsaved`, named by the next count whose file is not `taken`, so it never replaces one.
    pub(super) fn name(
        &mut self,
        unsaved: &FolderPath,
        taken: impl Fn(&SnipFile) -> bool,
    ) -> Result<SnipFile, RecordError> {
        loop {
            self.made = self.made.checked_add(1).ok_or(RecordError::Failed)?;
            let path = unsaved.expose().join(self.made.to_string());
            let path = path.with_extension(FileKind::Mp4.ext());
            let file = SnipFile::new(path, FileKind::Mp4).ok_or(RecordError::Failed)?;
            if !taken(&file) {
                return Ok(file);
            }
        }
    }
}

/// The log, and the signal each change gives, for a test waiting on another thread.
pub(super) struct Shared {
    pub(super) log: Mutex<Log>,
    pub(super) changed: Condvar,
}

impl Shared {
    /// Tells recording `id` `note` on this thread with the log unlocked; false when it hears no more. Its callback
    /// stays held from the marking to the telling, so notes keep their order across threads.
    pub(super) fn tell(&self, id: usize, note: RecordNote, folder: &FakeSnipFolder) -> bool {
        let hearer = lock(&self.log).runs[id].hearer.clone();
        let Some(hearer) = hearer else {
            return false;
        };
        let tell = lock(&hearer);
        let heard = self.mark(id, note, folder);
        if heard {
            tell(note);
        }
        heard
    }

    /// Logs `note` on recording `id`: a `Done` ends it and keeps its video at that length, a `Fault` ends it and
    /// deletes the video, and a `Lost` sound is heard off from now; false when it no longer hears.
    fn mark(&self, id: usize, note: RecordNote, folder: &FakeSnipFolder) -> bool {
        let now = folder.now();
        let mut log = lock(&self.log);
        let run = &mut log.runs[id];
        if !run.log.live {
            return false;
        }
        run.log.told.push(note);
        match note {
            RecordNote::Time { .. } => {}
            RecordNote::Lost(source) => {
                match source {
                    Source::Mic => run.now.mic = false,
                    Source::Sound => run.now.sound = false,
                }
                run.now.ms = ms(run.at, now);
                run.log.heard.push(run.now);
            }
            RecordNote::Done(done) => {
                run.end();
                let length_ms = done.length_ms;
                folder.put(run.log.file.clone(), Stored::Video { length_ms });
            }
            RecordNote::Fault(_) => {
                run.end();
                folder.forget(&run.log.file);
            }
        }
        drop(log);
        self.changed.notify_all();
        true
    }
}
