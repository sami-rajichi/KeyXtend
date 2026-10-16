//! A fake OS that records the host's calls, and a scripted engine that records what it heard.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};

use kx_module_api::Mono;
use kx_module_api::ports::{Point, PointerAct, PointerEngine, PointerEvent};

use crate::hook::Os;
use crate::hook::batch::point;
use crate::hook::inbox::Inbox;
use crate::keys::replay_tag;

/// The most acts the scripted engine pushes in one call.
pub(super) const MAX_ACTS: usize = 4;
/// The DPI the fake screens report.
pub(super) const DPI: u32 = 144;

/// One OS call the host made, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Call {
    /// A batch went out.
    Send(Vec<PointerAct>),
    /// The deadline timer was set, or stopped.
    Arm(Option<u32>),
    /// A wake was posted.
    Wake,
    /// Both hooks went in.
    Hook,
    /// Both hooks came out.
    Unhook,
}

/// What the fake OS answers, and the calls it heard.
#[derive(Debug)]
pub(super) struct World {
    pub calls: RefCell<Vec<Call>>,
    /// The platform clock, in µs.
    pub now: Cell<u64>,
    /// Windows' tick count, in ms.
    pub tick: Cell<u32>,
    pub last_input: Cell<Option<u32>>,
    /// Every point is on one of our windows.
    pub ours: Cell<bool>,
    /// How many acts each send takes; `None` takes them all.
    pub takes: Cell<Option<usize>>,
    /// The input desktop is not ours.
    pub away: Cell<bool>,
    /// The next hook install is refused with this code.
    pub refuse: Cell<Option<i32>>,
    /// Setting the deadline timer panics, as a fault outside the engine would.
    pub panic_arm: Cell<bool>,
    /// Setting the deadline timer is refused.
    pub refuse_timer: Cell<bool>,
    /// Sending panics once the echoes heard inside the call are in, as a fault inside `SendInput` would.
    pub panic_send: Cell<bool>,
    /// Each act the OS took, with its id, oldest first; an echo takes its own out.
    pub ids: RefCell<Vec<(PointerAct, u16)>>,
    /// Each send echoes the acts it took inside the call, in Windows' order, as our hooks then hear them.
    pub inside: Cell<bool>,
    /// Echoes of these ids are heard inside each send, before its own.
    pub strays: RefCell<Vec<u16>>,
    /// The next send that echoes inside the call drops its echo at this place in Windows' order, wrapping.
    pub lose: Cell<Option<usize>>,
    /// The hand's events heard inside each send; they pass without the host.
    pub hand_inside: Cell<usize>,
    /// Both hooks are in, so they hear what a send echoes inside the call.
    pub hooked: Cell<bool>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            calls: RefCell::default(),
            now: Cell::new(1_000_000),
            tick: Cell::new(1_000),
            last_input: Cell::new(Some(1_000)),
            ours: Cell::new(false),
            takes: Cell::new(None),
            away: Cell::new(false),
            refuse: Cell::new(None),
            panic_arm: Cell::new(false),
            refuse_timer: Cell::new(false),
            panic_send: Cell::new(false),
            ids: RefCell::default(),
            inside: Cell::new(false),
            strays: RefCell::default(),
            lose: Cell::new(None),
            hand_inside: Cell::new(0),
            hooked: Cell::new(false),
        }
    }
}

impl World {
    /// The calls so far, and forgets them.
    pub(super) fn take(&self) -> Vec<Call> {
        std::mem::take(&mut self.calls.borrow_mut())
    }

    /// Moves the clock and the tick count on by `ms`.
    pub(super) fn pass(&self, ms: u32) {
        self.now.set(self.now.get() + u64::from(ms) * 1_000);
        self.tick.set(self.tick.get().wrapping_add(ms));
    }

    fn note(&self, call: Call) {
        self.calls.borrow_mut().push(call);
    }
}

/// The fake OS: the host owns it, the test keeps the world.
pub(super) struct Fake(pub Rc<World>);

impl Os for Fake {
    fn now(&self) -> Mono {
        Mono::from_us(self.0.now.get())
    }

    fn tick(&self) -> u32 {
        self.0.tick.get()
    }

    fn last_input(&self) -> Option<u32> {
        self.0.last_input.get()
    }

    fn ours(&self, _at: Point) -> bool {
        self.0.ours.get()
    }

    fn dpi(&self, _at: Point) -> u32 {
        DPI
    }

    fn away(&self) -> bool {
        self.0.away.get()
    }

    fn wake(&self) -> bool {
        self.0.note(Call::Wake);
        true
    }

    /// Like Windows, input the OS took moves the last input time.
    fn send(&self, acts: &[PointerAct], first: u16, inbox: &mut Inbox) -> usize {
        self.0.note(Call::Send(acts.to_vec()));
        let took = self.0.takes.get().map_or(acts.len(), |n| n.min(acts.len()));
        if took > 0 {
            self.0.last_input.set(Some(self.0.tick.get()));
        }
        let ids = std::iter::successors(Some(first), |id| Some(id.wrapping_add(1)));
        let taken: Vec<_> = acts.iter().copied().zip(ids).take(took).collect();
        if self.0.hooked.get() {
            for &id in self.0.strays.borrow().iter() {
                inbox.hear(replay_tag(id));
            }
            (0..self.0.hand_inside.get()).for_each(|_| inbox.user(false, 0));
        }
        if self.0.inside.get() && self.0.hooked.get() && !taken.is_empty() {
            echo_inside(&taken, inbox, self.0.lose.take());
        }
        assert!(!self.0.panic_send.get(), "the fake send panics");
        self.0.ids.borrow_mut().extend(taken);
        took
    }

    fn arm(&self, ms: Option<u32>) -> bool {
        assert!(!self.0.panic_arm.get(), "the fake timer panics");
        self.0.note(Call::Arm(ms));
        ms.is_none() || !self.0.refuse_timer.get()
    }

    fn hook(&self) -> Result<(), i32> {
        self.0.note(Call::Hook);
        let answer = self.0.refuse.take().map_or(Ok(()), Err);
        self.0.hooked.set(answer.is_ok());
        answer
    }

    fn unhook(&self) {
        self.0.note(Call::Unhook);
        self.0.hooked.set(false);
    }

    fn hooked(&self) -> bool {
        self.0.hooked.get()
    }
}

/// Echoes `taken` into `inbox` as Windows does inside `SendInput`: the move of each pointed act, then each button and
/// key, in the batch's order. The echo at place `lose`, wrapping, never reaches the inbox.
fn echo_inside(taken: &[(PointerAct, u16)], inbox: &mut Inbox, lose: Option<usize>) {
    let moves = taken.iter().filter(|(act, _)| point(*act).is_some());
    let rest = taken
        .iter()
        .filter(|(act, _)| !matches!(act, PointerAct::Move(_)));
    let ids: Vec<u16> = moves.chain(rest).map(|&(_, id)| id).collect();
    let gone = lose.map(|k| k % ids.len());
    let kept = ids.iter().enumerate().filter(|&(k, _)| gone != Some(k));
    for (_, &id) in kept {
        inbox.hear(replay_tag(id));
    }
}

/// What the scripted engine heard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Heard {
    On(PointerEvent, Mono),
    Settle(Mono),
    Release,
    /// The host dropped the engine.
    Dropped,
}

/// What the scripted engine does, and what it heard.
#[derive(Debug, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one knob a test turns on the scripted engine."
)]
pub(super) struct Plan {
    /// `on` drops every event.
    pub drop: bool,
    /// `on` drops each release, as a carry drops the release of the press that picked up.
    pub drop_ups: bool,
    /// What each call pushes.
    pub on_acts: Vec<PointerAct>,
    pub settle_acts: Vec<PointerAct>,
    pub release_acts: Vec<PointerAct>,
    pub deadline: Option<Mono>,
    /// `on` moves the deadline here.
    pub arm_on: Option<Mono>,
    /// The engine says it holds notes for the module.
    pub notes: bool,
    /// `on` and `release` panic.
    pub panic: bool,
    pub heard: Vec<Heard>,
}

impl Plan {
    /// True when `on` drops an event, a release when `up`.
    pub(super) fn drops(&self, up: bool) -> bool {
        self.drop || (self.drop_ups && up)
    }
}

/// An engine that follows its plan.
pub(super) struct Script(pub Arc<Mutex<Plan>>);

impl Script {
    fn plan(&self) -> std::sync::MutexGuard<'_, Plan> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl PointerEngine for Script {
    fn on(&mut self, event: PointerEvent, now: Mono, acts: &mut Vec<PointerAct>) -> bool {
        let mut plan = self.plan();
        plan.heard.push(Heard::On(event, now));
        assert!(!plan.panic, "the scripted engine panics in on");
        if let Some(due) = plan.arm_on {
            plan.deadline = Some(due);
        }
        acts.extend_from_slice(&plan.on_acts);
        plan.drops(matches!(event, PointerEvent::Up { .. }))
    }

    fn settle(&mut self, now: Mono, acts: &mut Vec<PointerAct>) {
        let mut plan = self.plan();
        plan.heard.push(Heard::Settle(now));
        acts.extend_from_slice(&plan.settle_acts);
    }

    fn deadline(&self) -> Option<Mono> {
        self.plan().deadline
    }

    fn has_notes(&self) -> bool {
        self.plan().notes
    }

    fn release(&mut self, acts: &mut Vec<PointerAct>) {
        let mut plan = self.plan();
        plan.heard.push(Heard::Release);
        assert!(!plan.panic, "the scripted engine panics in release");
        acts.extend_from_slice(&plan.release_acts);
    }

    fn max_acts(&self) -> usize {
        MAX_ACTS
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        self.plan().heard.push(Heard::Dropped);
    }
}
