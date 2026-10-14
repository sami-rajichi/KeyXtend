//! The world the properties drive: Windows' one input stream, which passes each event through our hooks in order, what
//! reached the apps, and a hand that lets go only of what it holds.

use std::collections::VecDeque;

use kx_module_api::ports::{Button, Point, PointerAct, Scan};

use super::super::fake::{Call, Fake, Plan};
use super::super::{CTRL, ESC, KEY_DOWN, KEY_UP, Rig, SHIFT, VK_ESC, mouse_msg, rig};
use super::{End, Ev, Fate, Gesture};
use crate::hook::host::Host;
use crate::hook::{Mark, Source};
use crate::keys::REPLAY;

/// The row the hand's events lie on; each takes a new column, so its point names it.
const HAND_Y: i32 = 1;
/// `VK_CONTROL` and `VK_SHIFT`.
const VK_CONTROL: u32 = 0x11;
const VK_SHIFT: u32 = 0x10;
/// Each key's virtual-key code.
const VKS: [(Scan, u32); 3] = [(ESC, VK_ESC), (CTRL, VK_CONTROL), (SHIFT, VK_SHIFT)];

/// What the hand holds.
#[derive(Debug, Default)]
struct Hand {
    held: Vec<Button>,
    ctrl: bool,
}

/// The host, its fake OS, the stream and the hand.
pub(super) struct Sim {
    rig: Rig,
    /// Events not through the hooks yet, oldest first.
    stream: VecDeque<Ev>,
    apps: Vec<Ev>,
    made: Vec<PointerAct>,
    fate: Vec<Fate>,
    hooked: bool,
    hand: Hand,
    /// A gesture that lands right after our next batch, as a hand acting while the host sends.
    slip: Option<Gesture>,
    room: (usize, usize, usize),
    grew: bool,
    ours_dropped: bool,
}

impl Sim {
    /// A started host with a scripted engine, an empty stream and a hand that holds nothing.
    pub(super) fn new() -> Self {
        let rig = rig(Source::Physical);
        let room = rig.host.room();
        Self {
            rig,
            stream: VecDeque::new(),
            apps: Vec::new(),
            made: Vec::new(),
            fate: Vec::new(),
            hooked: true,
            hand: Hand::default(),
            slip: None,
            room,
            grew: false,
            ours_dropped: false,
        }
    }

    /// The engine's plan.
    pub(super) fn plan(&self) -> std::sync::MutexGuard<'_, Plan> {
        self.rig.plan()
    }

    /// The OS takes this many acts of each batch, or all.
    pub(super) fn takes(&self, n: Option<usize>) {
        self.rig.world.takes.set(n);
    }

    /// The hand's gesture enters the stream now.
    pub(super) fn gesture(&mut self, g: Gesture) {
        for act in self.acts_of(g) {
            self.push_hand(act);
        }
    }

    /// The gesture lands right after our next batch.
    pub(super) fn slip(&mut self, g: Gesture) {
        self.slip = Some(g);
    }

    /// The next event goes through the hooks, or passes them by when `seen` is false.
    pub(super) fn step(&mut self, seen: bool) {
        let Some(ev) = self.stream.pop_front() else {
            return;
        };
        if !self.hooked || !seen || self.through(ev) {
            self.apps.push(ev);
        }
        self.collect();
    }

    /// The stream catches up, then `ms` pass.
    pub(super) fn pass(&mut self, ms: u32) {
        self.drain();
        self.rig.world.pass(ms);
    }

    /// Runs a host call on the loop side.
    pub(super) fn call(&mut self, f: impl FnOnce(&mut Host<Fake>)) {
        f(&mut self.rig.host);
        self.collect();
    }

    /// The stream catches up, a last turn goes out whole, then the host stops, the rest reaches the apps and the host
    /// is dropped.
    pub(super) fn end(mut self) -> End {
        self.slip = None;
        self.drain();
        self.takes(None);
        self.call(Host::turn);
        self.drain();
        let settled = self.apps.len();
        self.call(Host::stop);
        self.drain();
        let Rig { host, plan, .. } = self.rig;
        drop(host);
        let heard = std::mem::take(
            &mut plan
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .heard,
        );
        End {
            apps: self.apps,
            settled,
            made: self.made,
            fate: self.fate,
            heard,
            grew: self.grew,
            ours_dropped: self.ours_dropped,
        }
    }

    fn drain(&mut self) {
        while !self.stream.is_empty() {
            self.step(true);
        }
    }

    /// The acts of `g`, as the hand stands now; flips change what it holds.
    fn acts_of(&mut self, g: Gesture) -> Vec<PointerAct> {
        let at = Point {
            x: i32::try_from(self.made.len()).unwrap_or(i32::MAX),
            y: HAND_Y,
        };
        match g {
            Gesture::Flip(b) if self.hand.held.contains(&b) => {
                self.hand.held.retain(|h| *h != b);
                vec![PointerAct::Up(b, at)]
            }
            Gesture::Flip(b) => {
                self.hand.held.push(b);
                vec![PointerAct::Down(b, at)]
            }
            Gesture::Move => vec![PointerAct::Move(at)],
            Gesture::Ctrl => {
                self.hand.ctrl = !self.hand.ctrl;
                vec![if self.hand.ctrl {
                    PointerAct::KeyDown(CTRL)
                } else {
                    PointerAct::KeyUp(CTRL)
                }]
            }
            Gesture::Esc => vec![PointerAct::KeyDown(ESC), PointerAct::KeyUp(ESC)],
        }
    }

    /// Puts the hand's `act` at the stream's end, as Windows' last input.
    fn push_hand(&mut self, act: PointerAct) {
        if !matches!(act, PointerAct::KeyDown(_) | PointerAct::KeyUp(_)) {
            self.made.push(act);
            self.fate.push(Fate::Kept);
        }
        self.stream.push_back(Ev { act, ours: false });
        let tick = self.rig.world.tick.get();
        self.rig.world.last_input.set(Some(tick));
    }

    /// Gives `ev` to its hook; true when it passes on.
    fn through(&mut self, ev: Ev) -> bool {
        let mark = Mark {
            injected: ev.ours,
            extra: if ev.ours { REPLAY } else { 0 },
            time: self.rig.world.tick.get(),
        };
        let dropped = match ev.act {
            PointerAct::KeyDown(scan) => {
                self.rig.host.key(KEY_DOWN, vk(scan), scan, mark);
                false
            }
            PointerAct::KeyUp(scan) => {
                self.rig.host.key(KEY_UP, vk(scan), scan, mark);
                false
            }
            act => {
                let (msg, at) = mouse_msg(act);
                let heard = self.rig.plan().heard.len();
                let dropped = self.rig.host.mouse(msg, at, mark);
                self.decided(ev, heard);
                dropped
            }
        };
        self.ours_dropped |= dropped && ev.ours;
        !dropped
    }

    /// Notes the hand's `ev` lost when the engine, which had heard `heard` events, dropped it.
    fn decided(&mut self, ev: Ev, heard: usize) {
        let plan = self.rig.plan();
        let dropped = plan.heard.len() > heard && plan.drop && !plan.panic;
        drop(plan);
        if dropped && !ev.ours {
            self.mark(ev.act, Fate::Lost);
        }
    }

    /// Notes what became of the hand's event `act`, if it is one; a lost event stays lost.
    fn mark(&mut self, act: PointerAct, fate: Fate) {
        let k = self.made.iter().position(|m| *m == act);
        if let Some(cell) = k.and_then(|k| self.fate.get_mut(k)) {
            *cell = if *cell == Fate::Lost {
                Fate::Lost
            } else {
                fate
            };
        }
    }

    /// Takes the host's OS calls: the taken part of each batch joins the stream, a slipped gesture lands after it, and
    /// the hooks go in or out.
    fn collect(&mut self) {
        for call in self.rig.world.take() {
            match call {
                Call::Send(acts) => {
                    let (takes, len) = (self.rig.world.takes.get(), acts.len());
                    let took = takes.map_or(len, |n| n.min(len));
                    let fate = if took < len { Fate::Cut } else { Fate::Kept };
                    for (i, act) in acts.iter().enumerate() {
                        self.mark(*act, if i < took { fate } else { Fate::Lost });
                    }
                    let sent = acts
                        .into_iter()
                        .take(took)
                        .map(|act| Ev { act, ours: true });
                    self.stream.extend(sent);
                    if let Some(g) = self.slip.take() {
                        self.gesture(g);
                    }
                }
                Call::Hook => self.hooked = true,
                Call::Unhook => self.hooked = false,
                Call::Arm(_) | Call::Wake => {}
            }
        }
        self.grew |= self.rig.host.room() != self.room;
    }
}

/// The virtual-key code of `scan`; 0 for a key the tests do not name.
fn vk(scan: Scan) -> u32 {
    VKS.iter()
        .find(|(s, _)| *s == scan)
        .map_or(0, |&(_, vk)| vk)
}
