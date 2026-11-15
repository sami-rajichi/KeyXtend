//! Properties over any run of clicks, notes, focus, session, layouts and time: the key is lit only while listening,
//! each run gets one Close and no call after it, no run stays open behind a hidden bar, rounds only grow, a hidden run
//! keeps and counts nothing, and a take once shown as dots is never shown or kept again.

use super::close::panel;
use super::*;
use kx_module_api::ports::SessionNote;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use std::collections::BTreeSet;

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 60;
/// The longest wait of one step, in ms, past the defaults' `done_ms` so bars fade.
const MAX_WAIT_MS: u64 = 12_000;
/// The keyboard languages a run moves between: two served, one to download, one no model knows.
const LANGS: [&str; 4] = ["fr", "en", "ar", "sw"];
/// How often the feature is turned on rather than off.
const ON: f64 = 0.8;
/// How often a note goes to the older take.
const OLD: f64 = 0.1;

/// The stages a run may tell.
const STAGES: [SpeechStage; 5] = [
    SpeechStage::Loading,
    SpeechStage::Open,
    SpeechStage::Shut,
    SpeechStage::Limit,
    SpeechStage::Silence,
];

/// The faults a run or a call may tell.
const FAULTS: [SpeechError; 19] = [
    SpeechError::Range("model"),
    SpeechError::BadPath,
    SpeechError::NoMic,
    SpeechError::MicBusy,
    SpeechError::MicRefused,
    SpeechError::Unplugged,
    SpeechError::NoFile,
    SpeechError::BadFile,
    SpeechError::Language,
    SpeechError::Runner,
    SpeechError::NoCard,
    SpeechError::CardRoom,
    SpeechError::CardFailed,
    SpeechError::NoSound,
    SpeechError::OutOfStep,
    SpeechError::Memory,
    SpeechError::Internal,
    SpeechError::Timeout,
    SpeechError::Gone,
];

/// What a run tells, by index into the tables above.
#[derive(Copy, Clone, Debug)]
enum Said {
    Stage(usize),
    Level(u8),
    Heard,
    Quiet,
    /// A piece at its place, empty when true.
    Piece(u16, bool),
    Done,
    Wiped,
    Fault(usize),
}

impl Said {
    fn note(self) -> SpeechNote {
        match self {
            Self::Stage(i) => SpeechNote::Stage(STAGES[i]),
            Self::Level(level) => SpeechNote::Level(level),
            Self::Heard => SpeechNote::Heard,
            Self::Quiet => SpeechNote::Quiet,
            Self::Piece(index, empty) => {
                let text = secret(if empty { " " } else { "mot" });
                SpeechNote::Piece { index, text }
            }
            Self::Done => SpeechNote::Done(done(1)),
            Self::Wiped => SpeechNote::Wiped,
            Self::Fault(i) => SpeechNote::Fault(FAULTS[i]),
        }
    }
}

/// One thing the flow meets.
#[derive(Clone, Debug)]
enum Step {
    Key,
    Bar(BarButton),
    /// A bar button clicked on the view this many rounds ago.
    Late(BarButton, u32),
    Again,
    Over(bool),
    Panel(bool),
    Field(FieldKind, u64),
    Elsewhere,
    Session(SessionNote),
    Layout(usize),
    /// The keyboard answers the last text ask, in full when true.
    Typed(bool),
    /// A note to the open take, or to the older one when true.
    Said(Said, bool),
    /// Time passes, in ms, then a tick.
    Wait(u64),
    On(bool),
    Quit,
}

fn said() -> impl Strategy<Value = Said> {
    prop_oneof![
        (0..STAGES.len()).prop_map(Said::Stage),
        any::<u8>().prop_map(Said::Level),
        Just(Said::Heard),
        Just(Said::Quiet),
        (0..3_u16, any::<bool>()).prop_map(|(i, empty)| Said::Piece(i, empty)),
        Just(Said::Done),
        Just(Said::Wiped),
        (0..FAULTS.len()).prop_map(Said::Fault),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    let button = || select(BarButton::ALL.to_vec());
    let kinds = [
        FieldKind::Text,
        FieldKind::Password,
        FieldKind::Unknown,
        FieldKind::None,
    ];
    prop_oneof![
        3 => Just(Step::Key),
        3 => button().prop_map(Step::Bar),
        1 => (button(), 1..4_u32).prop_map(|(b, n)| Step::Late(b, n)),
        1 => Just(Step::Again),
        1 => any::<bool>().prop_map(Step::Over),
        1 => any::<bool>().prop_map(Step::Panel),
        1 => (select(kinds.to_vec()), 0..2_u64).prop_map(|(k, b)| Step::Field(k, b)),
        1 => Just(Step::Elsewhere),
        1 => select(SessionNote::ALL.to_vec()).prop_map(Step::Session),
        1 => (0..LANGS.len()).prop_map(Step::Layout),
        1 => any::<bool>().prop_map(Step::Typed),
        6 => (said(), prop::bool::weighted(OLD)).prop_map(|(s, old)| Step::Said(s, old)),
        2 => (0..MAX_WAIT_MS).prop_map(Step::Wait),
        1 => prop::bool::weighted(ON).prop_map(Step::On),
        1 => Just(Step::Quit),
    ]
}

/// Feeds `step` to the rig; `ask` is the last text ask, which `Typed` answers.
fn act(rig: &mut Rig, step: &Step, ask: Option<&TextAsk>) -> Vec<Out> {
    match step {
        Step::Key => rig.key(),
        Step::Bar(button) => rig.bar(*button),
        Step::Late(button, back) => {
            let round = rig.round().saturating_sub(*back);
            let click = VoiceClick::Bar(*button);
            rig.on(In::Act(VoiceAct { round, click }))
        }
        Step::Again => rig.again(),
        Step::Over(over) => rig.click(VoiceClick::Over(*over)),
        Step::Panel(open) => rig.on(In::Panel(open.then(|| Box::new(panel())))),
        Step::Field(kind, box_id) => rig.field(*kind, *box_id),
        Step::Elsewhere => rig.on(In::Elsewhere),
        Step::Session(note) => rig.on(In::Session(*note)),
        Step::Layout(i) => rig.on(In::Layout(layout(LANGS[*i], LANGS[*i], false))),
        Step::Typed(full) => ask.map_or_else(Vec::new, |ask| answer(rig, ask, *full)),
        Step::Said(said, old) => {
            let take = rig.take.wrapping_sub(u32::from(*old));
            let note = said.note();
            rig.on(In::Note { take, note })
        }
        Step::Wait(ms) => rig.tick_after(*ms),
        Step::On(on) => {
            let mut settings = VoiceSettings::from_defaults().unwrap();
            settings.on = *on;
            rig.on(In::Settings(Box::new(settings)))
        }
        Step::Quit => rig.on(In::Quit),
    }
}

/// The keyboard answers `ask`: all typed when `full`, else nothing and a failure.
fn answer(rig: &mut Rig, ask: &TextAsk, full: bool) -> Vec<Out> {
    let count = u32::try_from(ask.text.expose().chars().count()).unwrap();
    let round = ask.round;
    let count = if full { count } else { 0 };
    rig.on(In::Typed(TextDone {
        round,
        count,
        ok: full,
    }))
}

/// What the run saw so far.
#[derive(Default)]
struct Seen {
    opened: BTreeSet<u32>,
    closed: BTreeSet<u32>,
    /// The takes shown as dots.
    masked: BTreeSet<u32>,
    round: u32,
    ask: Option<TextAsk>,
}

impl Seen {
    /// Checks one step's effects; a hidden run keeps and counts nothing.
    fn effects(&mut self, out: &[Out], hidden: bool) -> Result<(), TestCaseError> {
        for effect in out {
            match effect {
                Out::Open { take, .. } => {
                    prop_assert!(self.opened.insert(*take), "{take} reopened");
                }
                Out::Run { take, call } => {
                    let live = self.opened.contains(take) && !self.closed.contains(take);
                    prop_assert!(live, "{call:?} on run {take}");
                    prop_assert!(
                        !hidden || *call != RunCall::Keep,
                        "a hidden run kept its sound"
                    );
                    if *call == RunCall::Close {
                        self.closed.insert(*take);
                    }
                }
                Out::View(view) => {
                    prop_assert!(view.0.round > self.round, "round {} again", view.0.round);
                    self.round = view.0.round;
                }
                Out::Type(ask) => self.ask = Some(ask.clone()),
                Out::Keep(kept) => {
                    prop_assert!(
                        !hidden && !self.masked.contains(&kept.take),
                        "kept {kept:?}"
                    );
                }
                Out::Count(_) => prop_assert!(!hidden, "a hidden run counted"),
                _ => {}
            }
        }
        Ok(())
    }

    /// Checks the bar and the light after a step.
    fn bar(&mut self, rig: &Rig) -> Result<(), TestCaseError> {
        let shown = rig.shown();
        prop_assert!(
            !rig.lit || matches!(shown, VoiceBar::Listening(_)),
            "lit on {shown:?}"
        );
        let closed = shown != VoiceBar::Hidden || self.opened == self.closed;
        prop_assert!(closed, "a run stays open behind the hidden bar");
        match &shown {
            VoiceBar::Text(text) if text.text == VoiceText::Masked => {
                self.masked.insert(rig.take);
            }
            VoiceBar::Text(_) => {
                prop_assert!(!self.masked.contains(&rig.take), "dots shown in plain");
            }
            VoiceBar::Note(note) => prop_assert!(!note.mic),
            _ => {}
        }
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn any_run_keeps_the_light_the_runs_and_the_texts_right(
        steps in vec(step(), 1..MAX_STEPS),
        hidden in any::<bool>(),
        keep_voice in any::<bool>(),
        silence_stop in any::<bool>(),
    ) {
        let mut settings = VoiceSettings::from_defaults().unwrap();
        settings.keep_voice = keep_voice;
        settings.silence_stop = silence_stop;
        let mut rig = Rig::with(settings, hidden);
        let mut seen = Seen::default();
        for step in &steps {
            let out = act(&mut rig, step, seen.ask.as_ref());
            seen.effects(&out, hidden)?;
            seen.bar(&rig)?;
        }
    }
}
