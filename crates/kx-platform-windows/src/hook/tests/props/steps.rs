//! The steps a random run takes, and how often each comes up in a calm, a flooded and a rough run.

use kx_module_api::ports::{Point, PointerAct};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

use super::super::fake::{MAX_ACTS, Plan};
use super::super::{CFG, CTRL, SHIFT};
use super::{BUTTONS, Gesture};
use crate::hook::queue::QUEUE_ROOM;

/// Most moves the hand makes at once, past what the queue holds.
const FLOOD: usize = QUEUE_ROOM;
/// Longest pause, past the health period, so lagging hooks go back.
const MAX_PASS_MS: u32 = 2 * CFG.health_ms;
/// Shortest stall: just past the wait for our echo, so a wait it holds up ends as lost.
const MIN_STALL_MS: u32 = CFG.echo_ms + 1;
/// Longest stall: two waits for our echo, so a newer batch's wait can end as lost too.
const MAX_STALL_MS: u32 = 2 * CFG.echo_ms;
/// Where the engine's own acts land, off the hand's row.
pub(super) const ENGINE_AT: Point = Point { x: -1, y: -1 };

/// One thing that happens.
#[derive(Clone, Debug)]
pub(super) enum Step {
    /// The hand acts, and its event joins the stream.
    Hand(Gesture),
    /// The hand acts while the host sends: its event lands right after our next batch.
    Slip(Gesture),
    /// The stream's next event goes through the hooks.
    Rit,
    /// The stream's next event passes the hooks by, as when Windows dropped them.
    Miss,
    /// The stream catches up, then time passes.
    Pass(u32),
    /// Time passes while the stream stalls, so what waits in it reaches the hooks late.
    Stall(u32),
    /// Our next event in the stream passes our hooks by, so its echo is lost.
    Swallow,
    /// Our hooks miss the echo at this place, wrapping, of the next send that echoes inside the call.
    Lose(usize),
    /// A wake or the deadline timer.
    Turn,
    /// The health timer.
    Check,
    /// The panic hook's let-go.
    Release,
    /// The OS takes this many acts of each batch, or all.
    Takes(Option<usize>),
    /// The engine follows a new plan.
    Plan(Script),
    /// The hand moves this many times at once.
    Flood(usize),
    /// The hand moves this many times at once, then acts, as when it lets go after a fast drag.
    Rush(usize, Gesture),
    /// The engine panics from now on, or stops.
    Panic(bool),
}

/// What the engine does with each call.
#[derive(Clone, Debug, Default)]
pub(super) struct Script {
    pub(super) drop: bool,
    /// Drops only releases, as a carry does.
    pub(super) carry: bool,
    pub(super) on: Vec<PointerAct>,
    pub(super) settle: Vec<PointerAct>,
    pub(super) release: Vec<PointerAct>,
    pub(super) notes: bool,
}

impl Script {
    /// Makes `plan` follow this script.
    pub(super) fn apply(&self, plan: &mut Plan) {
        plan.drop = self.drop;
        plan.drop_ups = self.carry;
        plan.on_acts.clone_from(&self.on);
        plan.settle_acts.clone_from(&self.settle);
        plan.release_acts.clone_from(&self.release);
        plan.notes = self.notes;
    }
}

fn gesture() -> impl Strategy<Value = Gesture> {
    prop_oneof![
        3 => select(BUTTONS.to_vec()).prop_map(Gesture::Flip),
        3 => Just(Gesture::Move),
        1 => Just(Gesture::Ctrl),
        1 => Just(Gesture::Esc),
    ]
}

fn act() -> impl Strategy<Value = PointerAct> {
    prop_oneof![
        Just(PointerAct::Move(ENGINE_AT)),
        select(BUTTONS.to_vec()).prop_map(|b| PointerAct::Down(b, ENGINE_AT)),
        select(BUTTONS.to_vec()).prop_map(|b| PointerAct::Up(b, ENGINE_AT)),
        select(vec![CTRL, SHIFT]).prop_map(PointerAct::KeyDown),
        select(vec![CTRL, SHIFT]).prop_map(PointerAct::KeyUp),
    ]
}

fn acts() -> impl Strategy<Value = Vec<PointerAct>> {
    vec(act(), 0..=MAX_ACTS)
}

fn script() -> impl Strategy<Value = Script> {
    let flags = (any::<bool>(), any::<bool>(), any::<bool>());
    (flags, acts(), acts(), acts()).prop_map(|((drop, carry, notes), on, settle, release)| Script {
        drop,
        carry,
        on,
        settle,
        release,
        notes,
    })
}

/// A plan that pushes acts at each call and each settle, so our input is often out and a flood fills the queue.
pub(super) fn busy() -> impl Strategy<Value = Step> {
    let some = || vec(act(), 1..=MAX_ACTS);
    (some(), some(), any::<bool>()).prop_map(|(on, settle, carry)| {
        let (drop, notes, release) = (false, false, Vec::new());
        Step::Plan(Script {
            drop,
            carry,
            on,
            settle,
            release,
            notes,
        })
    })
}

/// Steps where the queue never fills and the engine never fails; an echo of ours may be swallowed or dropped.
pub(super) fn calm() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => gesture().prop_map(Step::Hand),
        1 => gesture().prop_map(Step::Slip),
        4 => Just(Step::Rit),
        1 => (0..=MAX_PASS_MS).prop_map(Step::Pass),
        2 => Just(Step::Turn),
        1 => Just(Step::Check),
        1 => Just(Step::Release),
        1 => prop_oneof![Just(None), (0..=MAX_ACTS).prop_map(Some)].prop_map(Step::Takes),
        1 => script().prop_map(Step::Plan),
        1 => Just(Step::Swallow),
        1 => any::<usize>().prop_map(Step::Lose),
    ]
}

/// A stall long enough that a wait it holds up ends as lost.
fn stall() -> impl Strategy<Value = Step> {
    (MIN_STALL_MS..=MAX_STALL_MS).prop_map(Step::Stall)
}

/// Calm steps with busy plans, floods, stalls, swallowed echoes and panics, where the hand's input may pass ahead of ours.
pub(super) fn flooded() -> impl Strategy<Value = Step> {
    prop_oneof![
        24 => calm(),
        4 => busy(),
        4 => (1..=FLOOD).prop_map(Step::Flood),
        4 => (FLOOD / 2..=FLOOD, gesture()).prop_map(|(n, g)| Step::Rush(n, g)),
        4 => stall(),
        3 => Just(Step::Swallow),
        1 => any::<bool>().prop_map(Step::Panic),
    ]
}

/// Calm steps with floods, stalls, missed events and panics.
pub(super) fn rough() -> impl Strategy<Value = Step> {
    prop_oneof![
        12 => calm(),
        1 => (1..=FLOOD).prop_map(Step::Flood),
        1 => stall(),
        1 => Just(Step::Miss),
        1 => any::<bool>().prop_map(Step::Panic),
    ]
}
