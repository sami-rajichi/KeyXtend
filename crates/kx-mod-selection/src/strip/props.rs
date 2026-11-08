//! Properties over any run of looks, keys, presses, aims, keyboard moves, acts, new settings and waits: a closed box
//! never shows chips, nothing shows while a panel line is aimed at, the keyboard is in the tray or a selection is put
//! away, rounds only grow, and the view sent always matches the strip's own.

use super::*;
use crate::chips::Rules;
use crate::settings::CARET_S;
use kx_module_api::US_PER_S;
use kx_module_api::ports::{Rect, Screen};
use proptest::collection::vec;
use proptest::option;
use proptest::prelude::*;
use proptest::sample::select;
use std::time::Duration;

/// Random runs per property.
const CASES: u32 = 512;
/// Most steps in one run.
const MAX_STEPS: usize = 60;
/// Box ids tried, few so that boxes meet again.
const BOXES: u64 = 3;
/// Rounds an act may name, so some hit and some miss.
const ROUNDS: u32 = 8;
/// A side of the small grid the boxes sit on, so that some spots repeat.
const GRID: i32 = 40;
/// How often a switch is on, so most runs show strips.
const ON: f64 = 0.8;

/// One thing the strip meets.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// An input.
    In(Input),
    /// A click on the live strip's `SelChip`, acted.
    ActLive(SelChip),
    /// The last look again, with the box now of this kind, as when a box turns into a password box.
    Turn(SelKind),
    /// New settings.
    Configure(StripConfig),
    /// Time passes, in µs, then a tick.
    Wait(u64),
}

fn rect() -> impl Strategy<Value = Rect> {
    (0..2_i32, 0..2_i32).prop_map(|(x, y)| Rect {
        x: x * GRID,
        y: y * GRID,
        w: GRID / 2,
        h: GRID / 2,
    })
}

fn spot() -> impl Strategy<Value = SelSpot> {
    let kind = select(SelKind::ALL.to_vec());
    let boxes = (option::of(rect()), option::of(rect()), option::of(rect()));
    (0..BOXES, kind, any::<bool>(), boxes).prop_map(
        |(box_id, kind, selected, (first, last, caret))| SelSpot {
            box_id,
            kind,
            selected,
            first,
            last,
            caret,
            screen: Screen::default(),
        },
    )
}

fn config() -> impl Strategy<Value = StripConfig> {
    let switch = proptest::bool::weighted(ON);
    (proptest::array::uniform7(switch), CARET_S).prop_map(|(b, caret_s)| StripConfig {
        rules: Rules {
            on: b[0],
            copy: b[1],
            cut: b[2],
            paste: b[3],
            translate: b[4],
            caret_paste: b[5],
        },
        minimised: b[6],
        caret: Duration::from_secs(caret_s.into()),
    })
}

fn input() -> impl Strategy<Value = Input> {
    let look = (spot(), any::<bool>(), any::<bool>()).prop_map(|(spot, click, clip)| Input::Look {
        spot,
        click,
        clip,
    });
    let chip = select(SelChip::ALL.to_vec());
    let acted = (0..=ROUNDS, chip).prop_map(|(round, chip)| Input::Acted { round, chip });
    let board = select(vec![Board::Open, Board::Minimised, Board::Tray]);
    prop_oneof![
        8 => look,
        1 => Just(Input::Lost),
        1 => Just(Input::Key),
        1 => Just(Input::Press),
        1 => Just(Input::Esc),
        2 => any::<bool>().prop_map(Input::Aimed),
        2 => board.prop_map(Input::Board),
        1 => acted,
        1 => Just(Input::Tick),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    let longest = u64::from(*CARET_S.end()) * u64::from(US_PER_S);
    prop_oneof![
        12 => input().prop_map(Step::In),
        2 => select(SelChip::ALL.to_vec()).prop_map(Step::ActLive),
        2 => select(SelKind::ALL.to_vec()).prop_map(Step::Turn),
        1 => config().prop_map(Step::Configure),
        2 => (0..=2 * longest).prop_map(Step::Wait),
    ]
}

/// What the run has told the strip so far, for the checks.
#[derive(Default)]
struct Told {
    /// A panel line is aimed at.
    aimed: bool,
    /// Where the keyboard is.
    board: Board,
    /// The last look's kind; `None` before any look and after a lost one.
    kind: Option<SelKind>,
    /// The last look's spot.
    spot: Option<SelSpot>,
    /// The round of the newest strip sent.
    top: u32,
    /// A tick was sent and no look came since, so no new strip may show.
    ticked: bool,
    /// The look the strip keeps; `None` after a lost look, a press, or a key that ended a put-away.
    last: Option<SelSpot>,
    /// The box whose selection Esc or an act put away, so no new strip may show.
    away: Option<u64>,
}

impl Told {
    /// Notes `input`.
    fn hear(&mut self, input: Input) {
        match input {
            Input::Look { spot, .. } => {
                (self.kind, self.spot, self.ticked) = (Some(spot.kind), Some(spot), false);
                if self
                    .away
                    .is_some_and(|b| b != spot.box_id || !spot.selected)
                {
                    self.away = None;
                }
                self.last = Some(spot);
            }
            Input::Lost => (self.kind, self.last) = (None, None),
            Input::Key => {
                if self.away.take().is_some() {
                    self.last = None;
                }
            }
            Input::Press => (self.last, self.away) = (None, None),
            Input::Esc => self.put_away(),
            Input::Aimed(on) => self.aimed = on,
            Input::Board(board) => self.board = board,
            _ => {}
        }
    }

    /// Notes that the last look's selection, if any, is put away.
    fn put_away(&mut self) {
        self.away = self.last.filter(|s| s.selected).map(|s| s.box_id);
    }

    /// Nothing may show.
    fn blocked(&self) -> bool {
        self.aimed || self.board == Board::Tray || self.kind == Some(SelKind::Closed)
    }
}

/// Feeds `step` to `strip` at `now`, noting it in `told`; the view sent, if any.
fn feed(strip: &mut Strip, told: &mut Told, step: Step, now: &mut Mono) -> Option<ChipsView> {
    let input = match step {
        Step::In(input) => input,
        Step::ActLive(chip) => match strip.view() {
            ChipsView::Shown { round, .. } => Input::Acted { round, chip },
            ChipsView::Hidden => Input::Tick,
        },
        Step::Turn(kind) => told.spot.map_or(Input::Tick, |spot| Input::Look {
            spot: SelSpot { kind, ..spot },
            click: false,
            clip: true,
        }),
        Step::Configure(cfg) => return strip.configure(cfg, *now),
        Step::Wait(us) => {
            *now = now.saturating_add(Duration::from_micros(us));
            Input::Tick
        }
    };
    told.hear(input);
    strip.step(input, *now)
}

/// The view the strip should hold after sending `sent` from `before`: an acted strip has ended.
fn after(before: ChipsView, sent: Option<&ChipsView>) -> ChipsView {
    match sent {
        None => before,
        Some(ChipsView::Shown { acted: None, .. }) => sent.cloned().unwrap_or(before),
        Some(_) => ChipsView::Hidden,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// No strip for a closed box, while blocked or while a selection is put away; each new strip's round passes every
    /// earlier one, a tick keeps the live round and no new strip follows it before a look, and the view kept is the
    /// one sent.
    #[test]
    fn closed_boxes_and_blocks_show_nothing_and_rounds_only_grow(
        cfg in config(),
        steps in vec(step(), 0..=MAX_STEPS),
    ) {
        let (mut strip, mut told, mut now) = (Strip::new(cfg), Told::default(), Mono::default());
        for step in steps {
            let before = strip.view();
            let sent = feed(&mut strip, &mut told, step, &mut now);
            if let Some(ChipsView::Shown { round, chips, acted, .. }) = &sent {
                prop_assert!(!told.blocked(), "{:?}", step);
                prop_assert!(told.kind.is_some(), "a strip needs a look");
                prop_assert!(!chips.is_empty());
                let order: Vec<_> = SelChip::ALL.into_iter().filter(|c| chips.contains(c)).collect();
                prop_assert_eq!(chips, &order);
                match acted {
                    None => {
                        prop_assert!(*round > told.top, "{} after {}", round, told.top);
                        prop_assert!(!told.ticked, "a new strip after a tick needs a look");
                        prop_assert!(told.away.is_none(), "a selection put away stays away");
                        told.top = *round;
                    }
                    Some(chip) => {
                        let live = matches!(before, ChipsView::Shown { round: r, .. } if r == *round);
                        prop_assert!(live, "a tick keeps the live round");
                        prop_assert!(chips.contains(chip));
                        told.ticked = true;
                        told.put_away();
                    }
                }
            }
            let want = after(before, sent.as_ref());
            prop_assert_eq!(strip.view(), want);
            if told.blocked() {
                prop_assert_eq!(strip.view(), ChipsView::Hidden, "{:?}", step);
            }
        }
    }
}
