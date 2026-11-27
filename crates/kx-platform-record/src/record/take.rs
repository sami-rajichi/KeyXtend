//! One take on one timeline: each slot obeys the job's orders, tells the time once a second, checks the room, takes the
//! newest frame and writes the sound heard up to a cushion behind now; a run that fell behind skips to now's slot, so
//! the picture never drifts from the sound. The end waits one cushion and writes the sound to the video's end.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use super::mix::{Mixer, place};
use super::{
    Film, MS_S, Order, Plan, RATE, RecordEnd, RecordFault, RecordNote, Studio, Switch, told_ms,
};

/// Nanoseconds in a second.
const NS_S: u64 = 1_000_000_000;

/// Why a take stopped without its video.
pub(super) enum Halt {
    /// A fault, told after the video is deleted.
    Fault(RecordFault),
    /// Cancel: the video is deleted and nothing is told.
    Cancel,
}

impl From<RecordFault> for Halt {
    fn from(fault: RecordFault) -> Self {
        Self::Fault(fault)
    }
}

/// A running take.
pub(super) struct Take<'a, S: Studio> {
    studio: &'a S,
    plan: &'a Plan,
    film: Film<S::Frame>,
    mix: Mixer,
    latest: Option<S::Frame>,
    t0: Instant,
    /// The next slot.
    n: u64,
    /// The slot from which the next time is told.
    tick: u64,
    fps: u64,
}

impl<'a, S: Studio> Take<'a, S> {
    /// A take of `plan` over `film`, with its sources in `mix`.
    pub(super) fn new(studio: &'a S, plan: &'a Plan, film: Film<S::Frame>, mix: Mixer) -> Self {
        let t0 = film.clock.now();
        Self {
            studio,
            plan,
            film,
            mix,
            latest: None,
            t0,
            n: 0,
            tick: 0,
            fps: u64::from(plan.fps.max(1)),
        }
    }

    /// Waits a slot at a time, at most `within`, for the first frame; the timeline starts once it came.
    pub(super) fn first(&mut self, within: Duration) -> Result<(), RecordFault> {
        let end = self.film.clock.now() + within;
        let slot = Duration::from_nanos(NS_S / self.fps);
        while self.latest.is_none() {
            let now = self.film.clock.now();
            if now >= end {
                return Err(RecordFault::Failed);
            }
            self.latest = self.film.screen.grab()?;
            if self.latest.is_none() {
                self.film.clock.until(now + slot)?;
            }
        }
        self.t0 = self.film.clock.now();
        self.mix.begin(self.t0);
        Ok(())
    }

    /// When slot `n` begins.
    fn due(&self, n: u64) -> Instant {
        self.t0 + Duration::from_nanos(n.saturating_mul(NS_S) / self.fps)
    }

    /// The first sound frame of slot `n`.
    fn heard_at(&self, n: u64) -> i64 {
        i64::try_from(n * u64::from(RATE) / self.fps).unwrap_or(i64::MAX)
    }

    /// Now's place on the timeline.
    fn here(&self) -> i64 {
        place(self.film.clock.now(), self.t0)
    }

    /// The video's length so far, in ms.
    fn length_ms(&self) -> u32 {
        told_ms(self.n * MS_S / self.fps)
    }

    /// Runs the slots until an order, the longest length or the floor ends them.
    pub(super) fn slots(
        &mut self,
        orders: &Receiver<Order>,
        tell: &mut dyn FnMut(RecordNote),
    ) -> Result<RecordEnd, Halt> {
        let cap = self.plan.cap();
        loop {
            self.film.clock.until(self.due(self.n))?;
            self.catch_up(cap);
            if self.n > 0
                && let Some(end) = self.obey(orders, tell)?
            {
                return Ok(end);
            }
            if self.n >= cap {
                return Ok(RecordEnd::Longest);
            }
            if self.n >= self.tick && !self.second(tell) {
                return Ok(RecordEnd::Full);
            }
            self.lost(tell);
            self.step()?;
            self.n += 1;
        }
    }

    /// Skips the slots that went by while the last one was written, never past `cap`; they hold the last frame.
    fn catch_up(&mut self, cap: u64) {
        let since = self.film.clock.now().saturating_duration_since(self.t0);
        let ticks = since.as_nanos() * u128::from(self.fps) / u128::from(NS_S);
        let now = u64::try_from(ticks).unwrap_or(u64::MAX);
        self.n = self.n.max(now.min(cap));
    }

    /// Takes the orders waiting; Stop, or the job gone, ends the take, and Cancel halts it.
    fn obey(
        &mut self,
        orders: &Receiver<Order>,
        tell: &mut dyn FnMut(RecordNote),
    ) -> Result<Option<RecordEnd>, Halt> {
        loop {
            match orders.try_recv() {
                Ok(Order::Switch(which, on)) => self.switch(which, on, tell),
                Ok(Order::Stop) | Err(TryRecvError::Disconnected) => {
                    return Ok(Some(RecordEnd::Stopped));
                }
                Ok(Order::Cancel) => return Err(Halt::Cancel),
                Err(TryRecvError::Empty) => return Ok(None),
            }
        }
    }

    /// Switches `which`, telling its ack, or its fault while the take goes on.
    fn switch(&mut self, which: Switch, on: bool, tell: &mut dyn FnMut(RecordNote)) {
        let id = self.plan.id;
        if on != self.mix.on(which) {
            if on {
                match self.studio.ear(which) {
                    Ok(ear) => self.mix.open(which, ear),
                    Err(fault) => return tell(RecordNote::Fault { id, fault }),
                }
            } else {
                self.mix.close(which, self.here());
            }
        }
        tell(RecordNote::Switched {
            id,
            switch: which,
            on,
        });
    }

    /// Checks the room and tells the whole second reached; false when the drive reached the floor.
    fn second(&mut self, tell: &mut dyn FnMut(RecordNote)) -> bool {
        let folder = self.plan.file.parent().unwrap_or(&self.plan.file);
        let floor = u64::from(self.plan.min_free_mb);
        if self.studio.room_mb(folder).is_some_and(|mb| mb < floor) {
            return false;
        }
        let whole = self.n / self.fps;
        self.tick = (whole + 1) * self.fps;
        let (id, ms) = (self.plan.id, told_ms(whole * MS_S));
        tell(RecordNote::Time { id, ms });
        true
    }

    /// Switches off each source whose stream stopped for good, telling it lost.
    fn lost(&mut self, tell: &mut dyn FnMut(RecordNote)) {
        for which in self.mix.lost() {
            self.mix.close(which, self.here());
            let id = self.plan.id;
            tell(RecordNote::Lost { id, switch: which });
        }
    }

    /// Writes slot `n`'s frame, then the sound heard up to a cushion behind now.
    fn step(&mut self) -> Result<(), RecordFault> {
        if let Some(frame) = self.film.screen.grab()? {
            self.latest = Some(frame);
        }
        let frame = self.latest.as_ref().ok_or(RecordFault::Failed)?;
        self.film.sink.frame(self.n, frame)?;
        let here = self.here();
        self.mix.pull(here);
        let upto = (here - self.mix.cushion()).min(self.heard_at(self.n + 1));
        self.write(upto)
    }

    /// Writes the sound up to place `upto`, a queue at a time, so a stall of minutes never takes its gap at once.
    fn write(&mut self, upto: i64) -> Result<(), RecordFault> {
        while let Some((first, pcm)) = self.mix.mix(upto) {
            self.film.sink.sound(first, &pcm)?;
        }
        Ok(())
    }

    /// Lets the last sound land, writes it to the video's last frame, ends the file and hands back `end`'s note.
    pub(super) fn finish(mut self, end: RecordEnd) -> Result<RecordNote, RecordFault> {
        let cushion = Duration::from_nanos(
            u64::try_from(self.mix.cushion()).unwrap_or(0) * NS_S / u64::from(RATE),
        );
        self.film.clock.until(self.due(self.n) + cushion)?;
        let here = self.here();
        self.mix.pull(here);
        self.write(self.heard_at(self.n))?;
        let (id, length_ms) = (self.plan.id, self.length_ms());
        self.film.sink.finish()?;
        Ok(RecordNote::End { id, length_ms, end })
    }
}
