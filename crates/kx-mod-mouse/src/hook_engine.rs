//! The engine as the hook runs it: the actor's commands wait in a queue until `settle`, and the engine's notes go
//! back to the actor. Nothing here blocks or calls back into the hook.

use crate::actor::Msg;
use crate::{Engine, MAX_ACTS, Note};
use kx_module_api::Mono;
use kx_module_api::ports::{Point, PointerAct, PointerEngine, PointerEvent, Scan, Toggle};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};

/// The most acts the commands one `settle` applies push together. Only a held or carried press makes a command
/// push input, as an event would, and the first such command lets go of it.
const CMD_ACTS: usize = MAX_ACTS;

/// A command for the engine from the actor.
pub(crate) enum Cmd {
    /// Flips a mode.
    Toggle(Toggle),
    /// Takes the keyboard's latched modifiers.
    Latches(Vec<Scan>),
    /// The pad starts or stops scrolling.
    Scroll(bool),
}

impl Cmd {
    /// A start for this command tells no one when the hook refuses it: the user aimed no mouse key at the hook.
    pub(crate) const fn quiet(&self) -> bool {
        matches!(self, Self::Latches(_) | Self::Scroll(_))
    }

    /// Applies the command to `engine`, which pushes any input to send into `acts`.
    fn apply(self, engine: &mut Engine, acts: &mut Vec<PointerAct>) {
        match self {
            Self::Toggle(which) => engine.toggle(which, acts),
            Self::Latches(held) => engine.set_latches(held),
            Self::Scroll(on) => engine.set_scrolling(on),
        }
    }
}

/// The latest pointer point and whether a move message waits for the actor, so moves coalesce.
#[derive(Default)]
pub(crate) struct Moves {
    /// The latest point, packed into one word so it never tears.
    at: AtomicU64,
    /// A move message is in flight.
    busy: AtomicBool,
}

impl Moves {
    /// Keeps `at` as the latest point; true when no move is in flight, so the caller sends one.
    fn put(&self, at: Point) -> bool {
        let packed =
            (u64::from(at.x.cast_unsigned()) << u32::BITS) | u64::from(at.y.cast_unsigned());
        self.at.store(packed, Ordering::SeqCst);
        !self.busy.swap(true, Ordering::SeqCst)
    }

    /// Ends the flight and gives the latest point; a move after this sends a new message.
    pub(crate) fn take(&self) -> Point {
        self.busy.store(false, Ordering::SeqCst);
        let packed = self.at.load(Ordering::SeqCst);
        let half = |bits: u64| u32::try_from(bits & u64::from(u32::MAX)).unwrap_or_default();
        Point {
            x: half(packed >> u32::BITS).cast_signed(),
            y: half(packed).cast_signed(),
        }
    }
}

/// The engine, its command queue and its way back to the actor.
pub(crate) struct HookEngine {
    engine: Engine,
    queue: Receiver<Cmd>,
    actor: Sender<Msg>,
    moves: Arc<Moves>,
    /// Reused for each `settle`'s notes.
    notes: Vec<Note>,
    /// How many queued commands `settle` applied.
    applied: u64,
    /// The `applied` count last reported idle, while the engine stays idle.
    idle_at: Option<u64>,
    /// The actor's number for the run this engine serves.
    run: u64,
}

impl HookEngine {
    /// Wraps a fresh `engine` for the actor's run number `run` and applies `first` to it at once, so the first event
    /// already sees it. A fresh engine holds nothing, so that command pushes no input.
    pub(crate) fn new(
        mut engine: Engine,
        first: Option<Cmd>,
        queue: Receiver<Cmd>,
        actor: Sender<Msg>,
        moves: Arc<Moves>,
        run: u64,
    ) -> Self {
        if let Some(cmd) = first {
            cmd.apply(&mut engine, &mut Vec::new());
        }
        Self {
            engine,
            queue,
            actor,
            moves,
            notes: Vec::new(),
            applied: 0,
            idle_at: None,
            run,
        }
    }

    /// Sends the engine's notes to the actor; a move goes only when none is in flight.
    fn tell(&mut self) {
        self.engine.take_notes(&mut self.notes);
        for note in self.notes.drain(..) {
            if let Note::Moved { at } = note
                && !self.moves.put(at)
            {
                continue;
            }
            // The actor outlives every run it starts, so only a stopping actor misses this.
            let _ = self.actor.send(Msg::Note(note));
        }
    }

    /// Tells the actor once when the engine no longer needs the hook, with the count of commands it has applied.
    fn check_idle(&mut self) {
        if self.engine.needs_hook() {
            self.idle_at = None;
        } else if self.idle_at != Some(self.applied) {
            self.idle_at = Some(self.applied);
            let _ = self.actor.send(Msg::Idle(self.applied));
        }
    }
}

impl PointerEngine for HookEngine {
    fn on(&mut self, event: PointerEvent, now: Mono, acts: &mut Vec<PointerAct>) -> bool {
        self.engine.on(event, now, acts)
    }

    /// A hold that came due fires before the commands, so input keeps its time order.
    fn settle(&mut self, now: Mono, acts: &mut Vec<PointerAct>) {
        self.engine.tick(now, acts);
        while let Ok(cmd) = self.queue.try_recv() {
            cmd.apply(&mut self.engine, acts);
            self.applied += 1;
        }
        self.tell();
        self.check_idle();
    }

    fn deadline(&self) -> Option<Mono> {
        self.engine.deadline()
    }

    /// Notes wait, or the engine went idle since its last idle report.
    fn has_notes(&self) -> bool {
        let idle = !self.engine.needs_hook() && self.idle_at != Some(self.applied);
        self.engine.has_notes() || idle
    }

    fn release(&mut self, acts: &mut Vec<PointerAct>) {
        self.engine.release(acts);
    }

    fn max_acts(&self) -> usize {
        MAX_ACTS + CMD_ACTS
    }
}

impl Drop for HookEngine {
    /// Tells the actor this run's engine is gone: the host drops it as the run stops, or once it gave up after a panic.
    fn drop(&mut self) {
        let _ = self.actor.send(Msg::Lost(self.run));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineCfg;
    use std::sync::mpsc;

    /// A hook engine with Right-click as given and no command waiting.
    fn hooked(rclick: bool) -> HookEngine {
        let (_, queue) = mpsc::channel();
        let (actor, _) = mpsc::channel();
        let cfg = EngineCfg {
            hold_us: 1,
            still_px: 1,
            rclick,
        };
        HookEngine::new(Engine::new(cfg), None, queue, actor, Arc::default(), 1)
    }

    #[test]
    fn a_move_with_a_mode_on_asks_for_a_turn_until_settled() {
        let (mut hook, mut acts) = (hooked(true), Vec::new());
        assert!(!hook.has_notes(), "nothing noted yet");
        let at = Point::default();
        assert!(!hook.on(PointerEvent::Move { at }, Mono::default(), &mut acts));
        assert!(hook.has_notes(), "the badge's move waits");
        hook.settle(Mono::default(), &mut acts);
        assert!(!hook.has_notes(), "told");
    }

    #[test]
    fn an_idle_engine_asks_for_a_turn_until_it_reported() {
        let (mut hook, mut acts) = (hooked(false), Vec::new());
        assert!(hook.has_notes(), "its idle report waits");
        hook.settle(Mono::default(), &mut acts);
        assert!(!hook.has_notes(), "reported once");
    }

    #[test]
    fn a_move_slot_keeps_the_latest_point_at_any_sign_and_size() {
        let moves = Moves::default();
        let edges = [i32::MIN, -1, 0, 1, i32::MAX];
        for (x, y) in edges.iter().flat_map(|&x| edges.map(|y| (x, y))) {
            moves.put(Point { x, y });
            assert_eq!(moves.take(), Point { x, y });
        }
    }

    #[test]
    fn a_move_in_flight_holds_back_the_next_until_taken() {
        let moves = Moves::default();
        let at = |x| Point { x, y: x };
        assert!(moves.put(at(1)), "the first move is sent");
        assert!(!moves.put(at(2)), "the second waits for the first");
        assert_eq!(moves.take(), at(2), "the actor gets the latest");
        assert!(moves.put(at(3)), "taken, so the next is sent");
    }
}
