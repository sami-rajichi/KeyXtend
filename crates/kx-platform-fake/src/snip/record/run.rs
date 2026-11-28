//! The fake's live recording: each switch checked as the port says and logged by the clock, Stop telling the end,
//! Cancel deleting the video, and a drop keeping it for the next start's sweep.

use super::gear::{mic_ready, sound_ready};
use super::log::{Hearer, Shared, ms};
use super::{Heard, RecordCall};
use crate::lock;
use crate::snip::folder::{FakeSnipFolder, Stored};
use kx_module_api::ports::{
    MicPass, MicSeal, RecordDone, RecordEnd, RecordError, RecordFacts, RecordNote, RecordRun,
    SnipFile,
};
use std::sync::Arc;

/// A live fake recording.
pub(super) struct FakeRecordRun {
    pub(super) shared: Arc<Shared>,
    pub(super) folder: FakeSnipFolder,
    /// The app's seal, which alone opens a microphone pass.
    pub(super) seal: MicSeal,
    /// Its place in the log's runs.
    pub(super) id: usize,
    pub(super) file: SnipFile,
}

/// Waits out a note being told to `hearer`, so none comes after.
fn quiet(hearer: Option<Hearer>) {
    if let Some(hearer) = hearer {
        drop(lock(&hearer));
    }
}

impl FakeRecordRun {
    /// Refuses `call` once the recording ended, then as `able` finds the devices, then as failed once; else logs
    /// `switch` now.
    fn switch(
        &self,
        call: RecordCall,
        able: impl FnOnce(RecordFacts) -> Result<(), RecordError>,
        switch: impl FnOnce(&mut Heard),
    ) -> Result<(), RecordError> {
        let now = self.folder.now();
        let mut log = lock(&self.shared.log);
        log.gone(self.id)?;
        able(log.facts)?;
        log.failed(call)?;
        let run = &mut log.runs[self.id];
        switch(&mut run.now);
        run.now.ms = ms(run.at, now);
        run.log.heard.push(run.now);
        drop(log);
        self.shared.changed.notify_all();
        Ok(())
    }
}

impl RecordRun for FakeRecordRun {
    fn file(&self) -> &SnipFile {
        &self.file
    }

    fn mic(&mut self, on: Option<MicPass>) -> Result<(), RecordError> {
        if on.as_ref().is_some_and(|pass| !self.seal.opens(pass)) {
            return Err(RecordError::BadPass);
        }
        let wanted = on.is_some();
        let able = |f: RecordFacts| if wanted { mic_ready(f.mic) } else { Ok(()) };
        self.switch(RecordCall::Mic, able, |h| h.mic = wanted)
    }

    fn sound(&mut self, on: bool) -> Result<(), RecordError> {
        let able = |f: RecordFacts| if on { sound_ready(f.sound) } else { Ok(()) };
        self.switch(RecordCall::Sound, able, |h| h.sound = on)
    }

    fn stop(&mut self) -> Result<(), RecordError> {
        let now = self.folder.now();
        let length_ms = {
            let mut log = lock(&self.shared.log);
            log.gone(self.id)?;
            log.failed(RecordCall::Stop)?;
            let run = &mut log.runs[self.id];
            run.log.stops += 1;
            ms(run.at, now)
        };
        let end = RecordEnd::Stopped;
        let done = RecordNote::Done(RecordDone { length_ms, end });
        self.shared.tell(self.id, done, &self.folder);
        Ok(())
    }

    /// Deletes the video of a live recording only: after a `Done` the port promises it is kept, so it stays.
    fn cancel(&mut self) {
        let hearer = {
            let mut log = lock(&self.shared.log);
            let run = &mut log.runs[self.id];
            run.log.cancels += 1;
            if run.log.live {
                self.folder.forget(&self.file);
            }
            run.end()
        };
        self.shared.changed.notify_all();
        quiet(hearer);
    }
}

impl Drop for FakeRecordRun {
    /// Stops a live recording and keeps its video at its length so far.
    fn drop(&mut self) {
        let now = self.folder.now();
        let hearer = {
            let mut log = lock(&self.shared.log);
            let run = &mut log.runs[self.id];
            run.log.dropped = true;
            if run.log.live {
                let length_ms = ms(run.at, now);
                self.folder
                    .put(self.file.clone(), Stored::Video { length_ms });
            }
            run.end()
        };
        self.shared.changed.notify_all();
        quiet(hearer);
    }
}
