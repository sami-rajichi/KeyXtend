//! Any run of clicks on the panel, late ones included, Compare's notes, a dictation and the panel opening again: a
//! dimmed button, stepper end, switch or download button never acts, a row click on rows that moved since is dropped,
//! the shown page stays within its list, Close closes, and Compare runs only on its page.

use super::super::options;
use super::super::rig::{Rig, card, row};
use super::*;
use crate::compare::Act;
use crate::flow::Lend;
use crate::settings::MODELS_DIR;
use kx_module_api::ports::{
    BarButton, OptionValue, SpeechDone, SpeechError, SpeechNote, SpeechStage,
};
use kx_module_api::{Redacted, Zeroizing};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

/// The set text Compare's runs read.
const SET: &str = "The evening train leaves at a quarter past seven.";
/// History's texts, more than a page holds.
const TEXTS: u32 = 7;
/// A card's free memory, roomy enough for a check.
const CARD_MB: u32 = 4_096;
/// The highest row place a click names, past a page's rows.
const MAX_ROW: u8 = 7;
/// The most rows a page the ui tells holds.
const MAX_PAGE_ROWS: u8 = 6;
/// The most rounds a late click lags.
const MAX_LAG: u32 = 4;
/// The most steps in one case.
const MAX_STEPS: usize = 60;

/// One thing the panel meets.
#[derive(Clone, Debug)]
enum Step {
    /// A click on the view drawn now.
    Click(VoiceClick),
    /// A click on the view this many rounds back, as a slow finger sends it.
    Late(VoiceClick, u32),
    /// A note to Compare's last run.
    Note(SpeechNote),
    /// A dictation runs, and Try again can act.
    Facts(bool, bool),
    /// The panel opens again.
    Open,
}

/// The notes Compare's run may tell.
fn notes() -> Vec<SpeechNote> {
    let done = SpeechDone {
        pieces: 1,
        sound_ms: 4_000,
        wait_ms: 900,
        load_ms: 0,
        memory_mb: 800,
        card: false,
    };
    let text = Redacted::new(Zeroizing::new(SET.to_owned()));
    vec![
        SpeechNote::Heard,
        SpeechNote::Level(1),
        SpeechNote::Stage(SpeechStage::Limit),
        SpeechNote::Piece { index: 0, text },
        SpeechNote::Done(done),
        SpeechNote::Wiped,
        SpeechNote::Fault(SpeechError::BadFile),
        SpeechNote::Fault(SpeechError::NoSound),
    ]
}

fn click() -> impl Strategy<Value = VoiceClick> {
    let set = |s: bool| VoiceClick::Compare(Redacted::new(if s { SET } else { "" }.to_owned()));
    let fetch = |(row, button)| VoiceClick::Fetch { row, button };
    prop_oneof![
        1 => (0..=MAX_PAGE_ROWS).prop_map(VoiceClick::Rows),
        3 => select(VoiceTab::ALL.to_vec()).prop_map(VoiceClick::Open),
        2 => any::<bool>().prop_map(|back| VoiceClick::Page { back }),
        4 => (0..MAX_ROW).prop_map(VoiceClick::Row),
        2 => (0..MAX_ROW, select(FetchButton::ALL.to_vec())).prop_map(fetch),
        1 => (0..MAX_ROW).prop_map(VoiceClick::Flip),
        2 => (0..MAX_ROW, any::<bool>()).prop_map(|(row, up)| VoiceClick::Step { row, up }),
        6 => select(PanelButton::ALL.to_vec()).prop_map(VoiceClick::Button),
        1 => any::<bool>().prop_map(set),
        1 => any::<bool>().prop_map(VoiceClick::Confirm),
        1 => select(vec![BarButton::Stop, BarButton::Cancel]).prop_map(VoiceClick::Bar),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        8 => click().prop_map(Step::Click),
        2 => (click(), 1..=MAX_LAG).prop_map(|(c, lag)| Step::Late(c, lag)),
        2 => select(notes()).prop_map(Step::Note),
        1 => (any::<bool>(), any::<bool>()).prop_map(|(d, a)| Step::Facts(d, a)),
        1 => Just(Step::Open),
    ]
}

/// What a click that does nothing leaves as it was.
#[derive(Debug, PartialEq)]
struct Shot {
    page: Option<Page>,
    at: usize,
    ticks: Vec<Key>,
    note: Option<BarHint>,
    asked: bool,
    offer: Option<String>,
    browse: Option<std::path::PathBuf>,
    lend: Lend,
    fetches: crate::fetch::Fetches,
}

fn shot(p: &Panel) -> Shot {
    Shot {
        page: p.page,
        at: p.at,
        ticks: p.ticks.clone(),
        note: p.note.clone(),
        asked: p.asked.is_some(),
        offer: p.offer.clone(),
        browse: p.browse.clone(),
        lend: p.compare.lend(),
        fetches: p.fetches.clone(),
    }
}

/// True when `view` draws no live control for `click`.
fn dimmed(click: &VoiceClick, view: &VoicePanel) -> bool {
    let at = |r: &u8| usize::from(*r);
    match (click, &view.sheet) {
        (VoiceClick::Button(b), _) => !view.acts.contains(b),
        (VoiceClick::Step { row, up }, VoiceSheet::Options(rows)) => {
            (rows.get(at(row))).is_none_or(|r| !r.live || at_end(r.value, *up))
        }
        (VoiceClick::Flip(row), VoiceSheet::Options(rows)) => (rows.get(at(row)))
            .is_none_or(|r| !r.live || !matches!(r.value, OptionValue::Switch(_))),
        (VoiceClick::Fetch { row, button }, VoiceSheet::Models(sheet)) => {
            let fetch = sheet.rows.get(at(row)).and_then(|r| r.fetch);
            fetch.is_none_or(|f| match button {
                FetchButton::Pause => f.paused,
                FetchButton::Resume => !f.paused,
                FetchButton::Cancel => false,
            })
        }
        (VoiceClick::Step { .. } | VoiceClick::Flip(_) | VoiceClick::Fetch { .. }, _) => true,
        _ => false,
    }
}

/// True when `value`'s minus (or plus when `up`) is dimmed, or it has none.
fn at_end(value: OptionValue, up: bool) -> bool {
    let ends = match value {
        OptionValue::Number { ends, .. }
        | OptionValue::Factor { ends, .. }
        | OptionValue::Choice { ends, .. } => ends,
        // The chosen chip picks nothing new.
        OptionValue::Pair { second, .. } => return second == up,
        OptionValue::Switch(_) | OptionValue::Hint => return true,
    };
    if up { ends.high } else { ends.low }
}

/// True for a click on a row, which names the rows it saw.
fn at_row(click: &VoiceClick) -> bool {
    matches!(
        click,
        VoiceClick::Row(_)
            | VoiceClick::Fetch { .. }
            | VoiceClick::Flip(_)
            | VoiceClick::Step { .. }
    )
}

/// What the case saw so far.
#[derive(Default)]
struct Seen {
    /// The round the drawn page or rows last changed.
    since: u32,
    face: (Option<Page>, Vec<Key>),
    /// Compare's last run.
    take: u32,
}

impl Seen {
    /// Notes the page and rows the rig drew last.
    fn look(&mut self, rig: &Rig) {
        let face = (rig.panel.page, rig.panel.shown.clone());
        if face != self.face {
            (self.face, self.since) = (face, rig.round);
        }
    }
}

/// Writes `[voice]` key `key` as the actor does, but the models folder, which this rig does not read again.
fn set(rig: &mut Rig, key: &str, value: toml::Value) {
    if key == MODELS_DIR {
        return;
    }
    let mut table = options::values(&rig.settings);
    table.insert(key.to_owned(), value);
    if let Ok(settings) = toml::Value::Table(table).try_into() {
        rig.settings = settings;
    }
}

/// Clicks `click` on the view of `round`; one that must do nothing, as `still` says, leaves the panel as it was.
fn press(
    rig: &mut Rig,
    click: &VoiceClick,
    round: u32,
    still: bool,
) -> Result<Vec<Do>, TestCaseError> {
    let before = still.then(|| shot(&rig.panel));
    let out = rig.click_at(round, click);
    if let Some(before) = before {
        prop_assert!(out.is_empty(), "{click:?} acted: {out:?}");
        prop_assert_eq!(before, shot(&rig.panel), "{:?} changed the panel", click);
    }
    Ok(out)
}

/// Runs `step` on the view the ui draws now; a closed panel takes no click.
fn run(rig: &mut Rig, seen: &mut Seen, step: &Step) -> Result<Vec<Do>, TestCaseError> {
    let view = rig.view();
    seen.look(rig);
    match (step, view) {
        (Step::Click(_) | Step::Late(..), None) => Ok(Vec::new()),
        (Step::Click(click), Some(view)) => press(rig, click, rig.round, dimmed(click, &view)),
        (Step::Late(click, lag), Some(_)) => {
            let round = rig.round.saturating_sub(*lag);
            press(rig, click, round, at_row(click) && round < seen.since)
        }
        (Step::Note(note), _) => Ok(rig.panel.compare_note(seen.take, note.clone())),
        (Step::Facts(dictating, again), _) => {
            if !dictating || !rig.panel.compare().busy() {
                rig.dictating = *dictating;
            }
            rig.again = *again && !rig.dictating;
            Ok(Vec::new())
        }
        (Step::Open, _) => Ok(rig.panel.open()),
    }
}

/// Checks the panel after a step's effects `out`, carried out as the actor does.
fn after(rig: &mut Rig, seen: &mut Seen, out: Vec<Do>) -> Result<(), TestCaseError> {
    let closed = out.contains(&Do::Close);
    for done in out {
        match done {
            Do::Compare(Act::Open { take, .. }) => seen.take = take,
            Do::Set(key, value) => set(rig, key, value),
            _ => {}
        }
    }
    let page = rig.panel.page();
    prop_assert!(!closed || page.is_none(), "Close left the panel open");
    let busy = rig.panel.compare().busy();
    let compare = page == Some(Page::Tab(VoiceTab::Compare));
    prop_assert!(!busy || compare, "Compare runs on {:?}", page);
    if let Some(view) = rig.view() {
        prop_assert!(view.pages.at < view.pages.count, "{:?}", view.pages);
        let per = rig.panel.rows.max(1);
        prop_assert!(rig.panel.shown.len() <= per, "more rows than a page holds");
    }
    seen.look(rig);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_dimmed_or_stale_click_never_acts_and_the_pages_hold(steps in vec(step(), 1..MAX_STEPS)) {
        let mut rig = Rig::foldered();
        rig.holds(&[row("parakeet-v3-q8"), row("whisper-small-q5km")]);
        rig.texts(TEXTS);
        rig.devices.push(card(CARD_MB));
        let mut seen = Seen::default();
        for step in &steps {
            let out = run(&mut rig, &mut seen, step)?;
            after(&mut rig, &mut seen, out)?;
        }
    }
}
