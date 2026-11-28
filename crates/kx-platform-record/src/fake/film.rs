//! The fake capture, file and clock: a frame is the time it was made, the file logs each frame and sound written and
//! exists on disk from its opening, and the clock jumps to each wait's end once the gate lets it.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::FakeStudio;
use crate::record::{CHANNELS, Clock, Film, Plan, RecordFault, Screen, Sink};

/// What the fake file holds once ended, so a test sees it was ended.
pub(super) const ENDED: &[u8] = b"ended";

/// Opens the fake capture, file and clock, as the script says; the file is made on disk.
pub(super) fn open(studio: &FakeStudio, plan: &Plan) -> Result<Film<Duration>, RecordFault> {
    let held = {
        let world = studio.world();
        world.script.film?;
        world.script.held
    };
    let made = if held {
        fs::create_dir(&plan.file)
    } else {
        fs::write(&plan.file, [])
    };
    made.map_err(|_| RecordFault::Failed)?;
    Ok(Film {
        screen: Box::new(FakeScreen(studio.clone())),
        sink: Box::new(FakeSink {
            studio: studio.clone(),
            file: plan.file.clone(),
        }),
        clock: Box::new(FakeClock(studio.clone())),
    })
}

/// The fake clock.
pub(super) struct FakeClock(pub(super) FakeStudio);

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        let world = self.0.world();
        world.base + world.elapsed
    }

    fn until(&mut self, at: Instant) -> Result<(), RecordFault> {
        self.0.pass();
        self.0.world().reach(at);
        Ok(())
    }
}

/// The fake capture: a new frame at every grab, once the script's first frame is due.
struct FakeScreen(FakeStudio);

impl Screen<Duration> for FakeScreen {
    fn grab(&mut self) -> Result<Option<Duration>, RecordFault> {
        let world = self.0.world();
        let now = world.elapsed;
        Ok((now >= world.script.frames_from).then_some(now))
    }
}

/// The fake file.
struct FakeSink {
    studio: FakeStudio,
    file: PathBuf,
}

impl Sink<Duration> for FakeSink {
    fn frame(&mut self, slot: u64, frame: &Duration) -> Result<(), RecordFault> {
        let mut world = self.studio.world();
        if world.script.sink_fails_at == Some(slot) {
            return Err(RecordFault::Failed);
        }
        if world.script.sink_panics_at == Some(slot) {
            drop(world);
            panic!("the scripted file panics at slot {slot}");
        }
        world.log.frames.push((slot, *frame));
        let stall = world.script.stall.filter(|(at, _)| *at == slot);
        let cost = world.script.frame_cost + stall.map_or(Duration::ZERO, |(_, long)| long);
        let done = world.base + world.elapsed + cost;
        world.reach(done);
        Ok(())
    }

    fn sound(&mut self, first: u64, pcm: &[i16]) -> Result<(), RecordFault> {
        self.studio.world().log.wrote(first, pcm);
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<(), RecordFault> {
        fs::write(&self.file, ENDED).map_err(|_| RecordFault::Failed)?;
        self.studio.world().log.finished = true;
        Ok(())
    }
}

/// The stereo frames in `pcm`.
pub(super) fn frames(pcm: &[i16]) -> u64 {
    u64::try_from(pcm.len() / usize::from(CHANNELS)).unwrap_or(u64::MAX)
}
