//! Any dictated text, erasing first, paced or not, met in its pauses by a click outside or on our windows, a latch, a
//! board key or a refused send: the Backspaces go first, nothing of the text goes after a click outside, a key sent or
//! a refusal, the answer counts exactly the characters that went and says yes only when all went, prediction hears one
//! break only when something went, and the text spends no latch.

use super::{BREAK, Heard, Mic, done};
use crate::held::A;
use crate::told::BKSP;
use kx_module_api::Bus;
use kx_module_api::ports::{FaceAction, InjectError, KeyId, KeyboardFace, PointerPressed, Scan};
use kx_platform_fake::{FakeInjector, Stroke};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use std::ops::Range;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

/// The characters a text is made of: no `a`, which the board key types, and Arabic among them.
const CHARS: [char; 8] = ['x', 'y', 'z', 'é', 'ع', 'م', '1', ' '];
/// The longest text, the most Backspaces and the longest gap an ask names.
const MAX_CHARS: usize = 12;
const MAX_ERASE: u32 = 4;
const MAX_GAP_MS: u16 = 50;
/// The most pauses a case scripts.
const MAX_PAUSES: usize = 16;
/// A send the app refuses whole.
const REFUSED: InjectError = InjectError::Refused { sent: 0, total: 1 };
/// The latch a case may hold.
const SHIFT: &str = "shift";

/// What meets the text inside one pause.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Meet {
    Nothing,
    /// A press outside our windows, or on them when false.
    Press(bool),
    Latch,
    Key,
    Refuse,
}

impl Meet {
    /// It ends the text: the caret may have moved, or the app refuses.
    fn stops(self) -> bool {
        matches!(self, Self::Press(true) | Self::Key | Self::Refuse)
    }
}

fn meet() -> impl Strategy<Value = Meet> {
    let all = [
        Meet::Nothing,
        Meet::Press(true),
        Meet::Press(false),
        Meet::Latch,
        Meet::Key,
        Meet::Refuse,
    ];
    select(all.to_vec())
}

/// What a pause may act on, as the mouse module, the UI and the app do.
#[derive(Clone)]
struct Hands {
    bus: Bus,
    face: Arc<dyn KeyboardFace>,
    injector: Arc<FakeInjector>,
}

impl Hands {
    fn meet(&self, meet: Meet) {
        let tap = |id: &str| {
            let id = KeyId::new(id);
            self.face.act(FaceAction::Tap { id, back: false }).unwrap();
        };
        match meet {
            Meet::Nothing => {}
            Meet::Press(outside) => {
                let bus = self.bus.clone();
                let told =
                    thread::spawn(move || bus.publish(&PointerPressed { ours: !outside }).is_ok());
                assert!(told.join().unwrap(), "the press was told");
            }
            Meet::Latch => tap(SHIFT),
            Meet::Key => tap(A),
            Meet::Refuse => self.injector.set_error(Some(REFUSED)),
        }
    }
}

/// Each pause's meeting, with the strokes before and after it.
type Log = Arc<Mutex<Vec<(Meet, Range<usize>)>>>;

/// Runs `meets` inside the pauses of `m`, one a pause, logging each.
fn script(m: &Mic, meets: Vec<Meet>) -> Log {
    let log = Log::default();
    let (seen, mut next) = (Arc::clone(&log), meets.into_iter());
    let hands = Hands {
        bus: m.b.bus.clone(),
        face: Arc::clone(&m.b.rig.face),
        injector: Arc::clone(&m.b.rig.fakes.injector),
    };
    m.b.rig.fakes.pause.during(move || {
        let Some(meet) = next.next() else { return };
        let before = hands.injector.strokes().len();
        hands.meet(meet);
        let after = hands.injector.strokes().len();
        seen.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((meet, before..after));
    });
    log
}

/// The text's own strokes: every stroke but those a pause's meeting made.
fn own(strokes: &[Stroke], log: &[(Meet, Range<usize>)]) -> Vec<Stroke> {
    let theirs = |i: &usize| log.iter().any(|(_, r)| r.contains(i));
    (0..strokes.len())
        .filter(|i| !theirs(i))
        .map(|i| strokes[i].clone())
        .collect()
}

/// The Backspaces at the front of `strokes`, and the text the rest typed; `None` when a Backspace or another stroke
/// comes after the text began.
fn split(strokes: &[Stroke]) -> Option<(u32, String)> {
    let bksp = Stroke::Tap(Scan::new(BKSP));
    let erased = strokes.iter().take_while(|s| **s == bksp).count();
    let mut typed = String::new();
    for s in &strokes[erased..] {
        let Stroke::Text(text, _) = s else {
            return None;
        };
        typed.push_str(text.expose());
    }
    Some((u32::try_from(erased).ok()?, typed))
}

/// What the text of `t` erasing `erase` sent among `strokes`: its Backspaces first, then a part of `t`, nothing after
/// a meeting that stops it, and all of it when none does.
fn sent(
    strokes: &[Stroke],
    log: &[(Meet, Range<usize>)],
    erase: u32,
    t: &str,
) -> Result<(u32, String), TestCaseError> {
    let mine = own(strokes, log);
    let split = split(&mine);
    prop_assert!(
        split.is_some(),
        "a Backspace after the text began: {:?}",
        mine
    );
    let (erased, typed) = split.unwrap_or_default();
    prop_assert!(
        erased <= erase && t.starts_with(&typed),
        "{erased} of {erase} erased"
    );
    prop_assert!(
        typed.is_empty() || erased == erase,
        "the text began before every Backspace"
    );
    if let Some((_, r)) = log.iter().find(|(meet, _)| meet.stops()) {
        prop_assert_eq!(
            strokes.len(),
            r.end,
            "the text went on after it was stopped"
        );
    } else {
        prop_assert!(
            erased == erase && typed == t,
            "an unstopped text did not all go"
        );
    }
    Ok((erased, typed))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_text_stops_where_focus_left_and_counts_what_went(
        chars in vec(select(CHARS.to_vec()), 0..=MAX_CHARS),
        erase in 0..=MAX_ERASE,
        gap_ms in prop_oneof![Just(0_u16), 1..=MAX_GAP_MS],
        quick_erase in any::<bool>(),
        latched in any::<bool>(),
        meets in vec(meet(), 0..MAX_PAUSES),
        round in 1..u32::MAX,
    ) {
        let m = Mic::quiet(quick_erase.then(|| ("erase_gap_ms", 0.into())));
        if latched {
            m.b.rig.tap(SHIFT).unwrap();
            m.heard();
        }
        let shift = m.b.key(SHIFT)["on"].clone();
        let log = script(&m, meets);
        let t: String = chars.into_iter().collect();
        m.ask(round, erase, &t, gap_ms);
        let strokes = m.b.rig.fakes.injector.strokes();
        let log = log.lock().unwrap_or_else(PoisonError::into_inner).clone();
        let (erased, typed) = sent(&strokes, &log, erase, &t)?;
        let count = u32::try_from(typed.chars().count()).unwrap_or(u32::MAX);
        let heard = m.heard();
        prop_assert_eq!(heard.last(), Some(&done(round, count, erased == erase && typed == t)));
        let answers = heard.iter().filter(|h| matches!(h, Heard::Done(_))).count();
        prop_assert_eq!(answers, 1, "one answer");
        let breaks = heard.iter().filter(|h| **h == BREAK).count();
        prop_assert_eq!(breaks, usize::from(erased > 0 || count > 0), "one break when something went");
        let touched = log.iter().any(|(meet, _)| matches!(meet, Meet::Latch | Meet::Key));
        prop_assert!(touched || m.b.key(SHIFT)["on"] == shift, "the text spent a latch");
    }
}
