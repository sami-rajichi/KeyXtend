//! The actor's counts, logged once as it quits: numbers only, never a word, an email or a typed text.

/// The message of the line logged with the counts when the actor quits.
pub const STOPPED_LINE: &str = "predict actor stopped";

/// What the actor did since it began.
#[derive(Debug, Default)]
pub(super) struct Stats {
    pub(super) pieces: u64,
    pub(super) reads: u64,
    pub(super) learned: u64,
    pub(super) emails: u64,
    /// Words asked of the keyboard.
    pub(super) sent: u64,
    /// Taps on an older lane.
    pub(super) stale: u64,
    /// Picks whose word the box no longer ended with.
    pub(super) moved: u64,
    /// Picks a box that cannot confirm the erase could not type.
    pub(super) unsure: u64,
    /// Picks past the keyboard's text cap.
    pub(super) long: u64,
    pub(super) forgot: u64,
    pub(super) switches: u64,
    pub(super) saves: u64,
    pub(super) unsaved: u64,
}

impl Stats {
    /// Logs every count in one line.
    pub(super) fn log(&self) {
        tracing::info!(
            pieces = self.pieces,
            reads = self.reads,
            learned = self.learned,
            emails = self.emails,
            sent = self.sent,
            stale = self.stale,
            moved = self.moved,
            unsure = self.unsure,
            long = self.long,
            forgot = self.forgot,
            switches = self.switches,
            saves = self.saves,
            unsaved = self.unsaved,
            "{STOPPED_LINE}"
        );
    }
}
