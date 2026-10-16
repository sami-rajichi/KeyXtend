//! Tests of the host over a fake OS: callbacks here, its limits in `limits`, the loop and the stop in `run`, the health
//! check in `health`, the queue in `order`, the wait for our echo in `echo` and for echoes heard inside our send in
//! `inside`, the record of held input in `held` and against the hand in `hand`, the other pure parts in `parts`, and
//! properties over random runs in `props`.

mod echo;
mod fake;
mod hand;
mod health;
mod held;
mod inside;
mod limits;
mod order;
mod parts;
mod props;
mod run;

use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use kx_module_api::ports::{Button, HookConfig, HookStats, Point, PointerAct, PointerEvent, Scan};
use kx_module_api::{Mono, US_PER_MS};

use super::host::Host;
use super::stats::Stats;
use super::{Mark, Source};
use crate::keys::{REPLAY, SIM_TAG, TAG, replay_tag};
use fake::{Call, DPI, Fake, Heard, Plan, Script, World};

/// A check every second with half a second of slack, and a quarter second for our input to come back.
const CFG: HookConfig = HookConfig {
    health_ms: 1_000,
    health_slack_ms: 500,
    echo_ms: 250,
};
/// `WM_MOUSEMOVE`, `WM_LBUTTONDOWN`, `WM_LBUTTONUP`, `WM_KEYDOWN`, `WM_KEYUP` and `VK_ESCAPE`.
const MOVE: u32 = 0x0200;
const LEFT_DOWN: u32 = 0x0201;
const LEFT_UP: u32 = 0x0202;
/// Each button's down and up message.
const BUTTONS: [(Button, u32, u32); 3] = [
    (Button::Left, LEFT_DOWN, LEFT_UP),
    (Button::Right, 0x0204, 0x0205),
    (Button::Middle, 0x0207, 0x0208),
];
const KEY_DOWN: u32 = 0x0100;
const KEY_UP: u32 = 0x0101;
const VK_ESC: u32 = 0x1B;
/// Another program's tag.
const OTHER: usize = 0x1234;
/// The bits of a mouse input's `dwExtraInfo` that Windows hands the hooks: the low 32 only; a key's keeps all 64.
const MOUSE_KEPT: usize = 0xFFFF_FFFF;
/// Left Shift and Left Ctrl, which the scripted engine pushes.
const SHIFT: Scan = Scan::new(0x2A);
const CTRL: Scan = Scan::new(0x1D);
/// The Esc key's scan code.
const ESC: Scan = Scan::new(0x01);
const A: Point = Point { x: 1, y: 2 };
const B: Point = Point { x: 3, y: 4 };

/// The platform clock at `n` ms.
fn ms(n: u64) -> Mono {
    Mono::from_us(n * u64::from(US_PER_MS))
}

/// A started host with its fake world, its engine's plan and its numbers.
struct Rig {
    host: Host<Fake>,
    world: Rc<World>,
    plan: Arc<Mutex<Plan>>,
    stats: Arc<Stats>,
}

fn rig(source: Source) -> Rig {
    let world = Rc::new(World::default());
    let plan = Arc::new(Mutex::new(Plan::default()));
    let stats = Arc::new(Stats::new());
    let engine = Box::new(Script(Arc::clone(&plan)));
    let fake = Fake(Rc::clone(&world));
    let mut host = Host::new(fake, engine, source, &CFG, Arc::clone(&stats));
    host.begin().unwrap();
    assert_eq!(world.take(), [Call::Hook]);
    Rig {
        host,
        world,
        plan,
        stats,
    }
}

impl Rig {
    fn plan(&self) -> MutexGuard<'_, Plan> {
        self.plan.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// What the engine heard so far, forgotten.
    fn heard(&self) -> Vec<Heard> {
        std::mem::take(&mut self.plan().heard)
    }

    fn stats(&self) -> HookStats {
        self.stats.snapshot()
    }

    fn now(&self) -> Mono {
        Mono::from_us(self.world.now.get())
    }

    /// A mark stamped now by Windows' tick count.
    fn mark(&self, injected: bool, extra: usize) -> Mark {
        Mark {
            injected,
            extra,
            time: self.world.tick.get(),
        }
    }

    /// The hand's mouse message `msg` at `at`; true when it is dropped.
    fn hand(&mut self, msg: u32, at: Point) -> bool {
        let mark = self.mark(false, 0);
        self.host.mouse(msg, at, mark)
    }

    /// An injected mouse message tagged `extra`, cut as Windows cuts it; true when it is dropped.
    fn injected(&mut self, msg: u32, at: Point, extra: usize) -> bool {
        let mark = self.mark(true, extra & MOUSE_KEPT);
        self.host.mouse(msg, at, mark)
    }

    /// Our sent `acts` come back through the hooks, each tagged with the id the oldest such act went out with.
    fn echo(&mut self, acts: &[PointerAct]) {
        for &act in acts {
            let mut ids = self.world.ids.borrow_mut();
            let k = ids.iter().position(|(sent, _)| *sent == act).unwrap();
            let (_, id) = ids.remove(k);
            drop(ids);
            let mark = self.mark(true, replay_tag(id));
            match act {
                PointerAct::KeyDown(scan) => self.host.key(KEY_DOWN, 0, scan, mark),
                PointerAct::KeyUp(scan) => self.host.key(KEY_UP, 0, scan, mark),
                act => {
                    let (msg, at) = mouse_msg(act);
                    assert!(!self.injected(msg, at, mark.extra), "our input passes");
                }
            }
        }
    }

    /// A turn that sends a move of ours, so later input waits for its echo.
    fn send_ours(&mut self) {
        self.plan().settle_acts = vec![PointerAct::Move(B)];
        self.host.turn();
        self.plan().settle_acts.clear();
        self.heard();
        self.world.take();
    }
}

/// The message and point of mouse act `act`.
fn mouse_msg(act: PointerAct) -> (u32, Point) {
    let msg = |b: Button, up: bool| {
        let row = BUTTONS.iter().find(|(x, ..)| *x == b);
        row.map_or(MOVE, |&(_, down, lift)| if up { lift } else { down })
    };
    match act {
        PointerAct::Down(b, at) => (msg(b, false), at),
        PointerAct::Up(b, at) => (msg(b, true), at),
        PointerAct::Move(at) => (MOVE, at),
        PointerAct::KeyDown(_) | PointerAct::KeyUp(_) => (0, Point::default()),
    }
}

#[test]
fn an_injected_event_without_our_tags_never_reaches_the_engine_and_passes() {
    let mut rig = rig(Source::Physical);
    rig.plan().drop = true;
    assert!(!rig.injected(MOVE, A, OTHER));
    assert!(!rig.injected(LEFT_DOWN, A, 0));
    assert!(rig.heard().is_empty());
    assert_eq!(rig.stats().events, 0);
}

#[test]
fn the_hand_reaches_the_engine_at_its_own_time() {
    let mut rig = rig(Source::Physical);
    let waited = 10;
    let mark = Mark {
        time: rig.world.tick.get() - waited,
        ..rig.mark(false, 0)
    };
    assert!(!rig.host.mouse(MOVE, A, mark));
    let at = Mono::from_us(rig.now().as_us() - u64::from(waited) * 1_000);
    assert_eq!(rig.heard(), [Heard::On(PointerEvent::Move { at: A }, at)]);
    assert_eq!((rig.stats().events, rig.stats().max_wait_ms), (1, waited));
}

#[test]
fn a_press_says_whether_it_landed_on_our_window_and_its_screens_dpi() {
    let mut rig = rig(Source::Physical);
    rig.world.ours.set(true);
    rig.hand(LEFT_DOWN, A);
    let down = PointerEvent::Down {
        button: Button::Left,
        at: A,
        ours: true,
        dpi: DPI,
    };
    assert_eq!(rig.heard(), [Heard::On(down, rig.now())]);
}

#[test]
fn in_simulated_mode_only_the_gate_tools_input_reaches_the_engine() {
    let mut rig = rig(Source::Simulated);
    rig.plan().drop = true;
    assert!(!rig.hand(MOVE, A), "the hand passes untouched");
    assert!(rig.heard().is_empty());
    assert!(rig.injected(MOVE, A, SIM_TAG));
    assert_eq!(rig.heard().len(), 1);
}

#[test]
fn the_engines_acts_queue_and_wake_the_thread_once_until_its_turn() {
    let mut rig = rig(Source::Physical);
    rig.plan().on_acts = vec![PointerAct::Up(Button::Right, A)];
    assert!(!rig.hand(LEFT_DOWN, A), "the engine let it through");
    assert!(rig.hand(MOVE, A), "behind the queued act");
    assert_eq!(rig.world.take(), [Call::Wake]);
    rig.host.turn();
    rig.hand(MOVE, A);
    assert!(
        rig.world.take().contains(&Call::Wake),
        "a new turn, a new wake"
    );
}

#[test]
fn a_moved_deadline_wakes_the_thread() {
    let mut rig = rig(Source::Physical);
    rig.plan().arm_on = Some(Mono::from_us(rig.now().as_us() + 1_500_000));
    rig.hand(LEFT_DOWN, A);
    assert_eq!(rig.world.take(), [Call::Wake]);
    rig.plan().arm_on = None;
    rig.host.turn();
    rig.world.take();
    rig.hand(MOVE, A);
    assert!(
        rig.world.take().is_empty(),
        "nothing queued, no new deadline"
    );
}

#[test]
fn notes_wake_the_thread_once_until_their_turn() {
    let mut rig = rig(Source::Physical);
    rig.plan().notes = true;
    for _ in 0..3 {
        assert!(!rig.hand(MOVE, A));
    }
    assert_eq!(
        rig.world.take(),
        [Call::Wake],
        "a fast mouse posts one wake"
    );
    rig.host.turn();
    let settled = Heard::Settle(rig.now());
    assert_eq!(
        rig.heard().last(),
        Some(&settled),
        "the turn takes the notes"
    );
    rig.world.take();
    rig.hand(MOVE, A);
    assert_eq!(rig.world.take(), [Call::Wake], "a new turn, a new wake");
}

#[test]
fn esc_from_the_hand_or_our_keyboard_reaches_the_engine_and_others_do_not() {
    let mut rig = rig(Source::Physical);
    rig.plan().drop = true;
    let marks = [(false, 0), (true, TAG), (true, OTHER), (true, REPLAY)];
    for (injected, extra) in marks {
        let mark = rig.mark(injected, extra);
        rig.host.key(KEY_DOWN, VK_ESC, ESC, mark);
    }
    let up = rig.mark(false, 0);
    rig.host.key(KEY_UP, VK_ESC, ESC, up);
    let esc = Heard::On(PointerEvent::Esc, rig.now());
    assert_eq!(rig.heard(), [esc.clone(), esc]);
}
