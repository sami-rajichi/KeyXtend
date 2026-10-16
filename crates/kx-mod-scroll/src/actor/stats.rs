//! The actor's counts, logged once as it quits: numbers only, never a point, a box or a target id.

use crate::Halt;

/// The message of the line logged with the counts when the actor quits.
pub const STOPPED_LINE: &str = "scroll actor stopped";

/// What the actor did since it began.
#[derive(Debug, Default)]
pub(super) struct Stats {
    pub(super) taps: u64,
    pub(super) ticks: u64,
    pub(super) turns: u64,
    /// Aims and turns the OS refused.
    pub(super) refused: u64,
    /// Stops by the Stop button or a third tap.
    user: u64,
    no_point: u64,
    nothing: u64,
    at_end: u64,
    end: u64,
    press: u64,
    limit: u64,
    gone: u64,
}

impl Stats {
    /// Counts a stop: by the user for none, else by `halt`'s cause.
    pub(super) fn stopped(&mut self, halt: Option<Halt>) {
        let count = match halt {
            None => &mut self.user,
            Some(Halt::NoPoint) => &mut self.no_point,
            Some(Halt::Nothing) => &mut self.nothing,
            Some(Halt::AtEnd) => &mut self.at_end,
            Some(Halt::End) => &mut self.end,
            Some(Halt::Press) => &mut self.press,
            Some(Halt::Limit) => &mut self.limit,
            Some(Halt::Gone) => &mut self.gone,
        };
        *count += 1;
    }

    /// Logs every count in one line.
    pub(super) fn log(&self) {
        tracing::info!(
            taps = self.taps,
            ticks = self.ticks,
            turns = self.turns,
            refused = self.refused,
            user = self.user,
            no_point = self.no_point,
            nothing = self.nothing,
            at_end = self.at_end,
            end = self.end,
            press = self.press,
            limit = self.limit,
            gone = self.gone,
            "{STOPPED_LINE}"
        );
    }
}
