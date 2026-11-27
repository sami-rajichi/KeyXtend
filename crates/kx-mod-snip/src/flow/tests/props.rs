//! Properties over any run of clicks, answers, recorder notes and time: no path keeps a file the user did not choose,
//! apart from a copied recording in `unsaved/`; the keyboard is back and the key dark whenever no snip runs; one snip
//! runs at a time and freezes once, after the keyboard's step aside is answered or timed out; a Copy or Save comes only
//! from its own click; Close asks exactly when nothing was kept; a region's part keeps to its frame; over frozen
//! screens three Escs at most end a snip; and no fault leaves a recording.

mod checks;
mod world;

use super::*;
use checks::{check_close, check_escs, check_frame, check_hidden};
use kx_module_api::ports::Handle;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use world::World;

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 80;
/// The longest wait of one step, in ms, past the countdown's second and the tick's time.
const MAX_WAIT_MS: u64 = 4_000;
/// The longest countdown a run picks, in seconds, so recordings are reached often.
const MAX_COUNT_S: u8 = 3;
/// The widest handle click area a run tells, in logical pixels, so a click near a handle may pick it up.
const MAX_HIT_PX: u16 = 8;
/// The clicks a run makes: the part's two corners, a pixel on the left screen and no window, and one on no screen.
const SPOTS: [Point; 4] = [NEAR, FAR, Point { x: 30, y: 25 }, AWAY];
/// The faults a run tells: a dead worker, and the listed devices that fail at the start.
const FAULTS: [RecordError; 4] = [
    RecordError::Gone,
    RecordError::NoMic,
    RecordError::MicRefused,
    RecordError::NoSound,
];

/// How a freeze answers.
#[derive(Copy, Clone, Debug)]
enum Froze {
    Through,
    TooBig,
    Refused,
}

/// What a recording's run tells.
#[derive(Copy, Clone, Debug)]
enum Said {
    Time,
    Done(RecordEnd),
    Fault(RecordError),
    Lost(Source),
}

/// One thing the flow meets.
#[derive(Clone, Debug)]
enum Step {
    Key,
    Esc,
    /// The ui's answer that the keyboard stepped aside.
    Aside,
    /// A capture window could not be kept out of captures.
    Bare,
    Freeze(Froze),
    Bar(BarKey),
    /// A click on the frozen screen, read by the part rules for the way picked.
    Spot(Point),
    /// The theme's handle click area across, in logical pixels.
    HitPx(u16),
    /// A reader's press on a frame's handle.
    Handle(Handle),
    Strip(StripKey),
    Record(RecKey),
    /// The newest open run tells this.
    Note(Said),
    /// A cancelled run's end that was already told: its video stays, and the flow hears it.
    Late,
    /// A cancelled run's time or lost sound that was already on its way.
    Stale(Said),
    /// New settings: the keyboard, the countdown and the microphone at start.
    Settings(SnipKeyboard, u8, bool),
    Edit(EditKey),
    Changed,
    Flat(Keep),
    Ask(AskKey),
    /// The Copy or Save asked answers: done, cancelled or failed.
    Answer(u8),
    /// The asked job starts writing its copy at a speed.
    Timing,
    Wait(Duration),
    Quit,
    /// A snip taken from the Snip key: a picture, or a video ready to record.
    Snap(bool),
    /// Record, then the longest countdown.
    Shoot,
    /// A picture taken from the Snip key, then Edit.
    Open,
}

fn said() -> impl Strategy<Value = Said> {
    let ends = vec![RecordEnd::Stopped, RecordEnd::Longest, RecordEnd::Full];
    prop_oneof![
        3 => Just(Said::Time),
        3 => select(ends).prop_map(Said::Done),
        1 => select(FAULTS.to_vec()).prop_map(Said::Fault),
        1 => select(vec![Source::Mic, Source::Sound]).prop_map(Said::Lost),
    ]
}

fn froze() -> impl Strategy<Value = Froze> {
    prop_oneof![
        8 => Just(Froze::Through),
        1 => Just(Froze::TooBig),
        1 => Just(Froze::Refused),
    ]
}

fn settings() -> impl Strategy<Value = Step> {
    let keyboard = select(SnipKeyboard::ALL.to_vec());
    (keyboard, 0..=MAX_COUNT_S, any::<bool>()).prop_map(|(k, c, m)| Step::Settings(k, c, m))
}

fn step() -> impl Strategy<Value = Step> {
    let stale = select(vec![Said::Time, Said::Lost(Source::Mic)]);
    prop_oneof![
        3 => Just(Step::Key),
        1 => Just(Step::Esc),
        2 => Just(Step::Aside),
        1 => Just(Step::Bare),
        3 => froze().prop_map(Step::Freeze),
        3 => select(BarKey::ALL.to_vec()).prop_map(Step::Bar),
        3 => select(SPOTS.to_vec()).prop_map(Step::Spot),
        1 => (0..=MAX_HIT_PX).prop_map(Step::HitPx),
        2 => select(Handle::ALL.to_vec()).prop_map(Step::Handle),
        4 => select(StripKey::ALL.to_vec()).prop_map(Step::Strip),
        3 => select(RecKey::ALL.to_vec()).prop_map(Step::Record),
        3 => said().prop_map(Step::Note),
        1 => Just(Step::Late),
        1 => stale.prop_map(Step::Stale),
        1 => settings(),
        2 => select(EditKey::ALL.to_vec()).prop_map(Step::Edit),
        2 => Just(Step::Changed),
        2 => select(Keep::ALL.to_vec()).prop_map(Step::Flat),
        2 => select(AskKey::ALL.to_vec()).prop_map(Step::Ask),
        3 => (0..3_u8).prop_map(Step::Answer),
        2 => Just(Step::Timing),
        2 => (0..MAX_WAIT_MS).prop_map(|ms| Step::Wait(Duration::from_millis(ms))),
        1 => Just(Step::Quit),
        3 => any::<bool>().prop_map(Step::Snap),
        2 => Just(Step::Shoot),
        2 => Just(Step::Open),
    ]
}

/// The plain steps `step` stands for, so runs reach recordings and the editor often.
fn plain(step: &Step) -> Vec<Step> {
    match step {
        Step::Snap(video) => {
            let way = if *video {
                BarKey::VideoRegion
            } else {
                BarKey::Region
            };
            let froze = Step::Freeze(Froze::Through);
            let take = [Step::Spot(NEAR), Step::Spot(FAR)];
            [Step::Key, Step::Aside, froze, Step::Bar(way)]
                .into_iter()
                .chain(take)
                .collect()
        }
        Step::Shoot => {
            let count = SECOND * u32::from(MAX_COUNT_S);
            vec![Step::Strip(StripKey::Record), Step::Wait(count)]
        }
        Step::Open => {
            let mut open = plain(&Step::Snap(false));
            open.push(Step::Strip(StripKey::Edit));
            open
        }
        other => vec![other.clone()],
    }
}

/// The freeze's answer; the flow drops one no freeze waits for.
fn freeze(froze: Froze) -> Result<Frozen, FreezeError> {
    match froze {
        Froze::Through => Ok(frozen()),
        Froze::TooBig => Err(FreezeError::TooBig),
        Froze::Refused => Err(FreezeError::Refused(5)),
    }
}

/// The defaults with `keyboard`, `countdown_s` and `mic_on` set.
fn settings_of(keyboard: SnipKeyboard, countdown_s: u8, mic_on: bool) -> SnipSettings {
    let mut settings = SnipSettings::from_defaults().unwrap();
    (settings.keyboard, settings.countdown_s, settings.mic_on) = (keyboard, countdown_s, mic_on);
    settings
}

/// Feeds one plain step and checks what follows.
fn one(world: &mut World, step: &Step) -> Result<(), TestCaseError> {
    let before = world.rig.stage().clone();
    let (free, kept) = (world.free(), world.kept());
    let outs = world.act(step)?;
    if free {
        check_close(world, step, &before, kept, &outs)?;
    }
    check_hidden(world, step, &before, &outs)?;
    check_frame(step, &before, world.rig.stage())?;
    let recording = matches!(world.rig.stage(), Stage::Record(_));
    prop_assert!(
        !(recording && matches!(step, Step::Note(Said::Fault(_)))),
        "a fault left it recording"
    );
    let waits = world.rig.flow.waits().filter(|_| world.free());
    prop_assert_eq!(waits, None, "a key waits with no job");
    world.check_idle()
}

/// The Copy or Save `step` clicks, if any.
fn clicked(step: &Step) -> Option<Keep> {
    match step {
        Step::Strip(StripKey::Copy) => Some(Keep::Copy),
        Step::Strip(StripKey::Save) | Step::Ask(AskKey::Save) => Some(Keep::Save),
        Step::Flat(keep) => Some(*keep),
        _ => None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn no_path_keeps_an_unchosen_file_or_leaves_the_keyboard_aside(
        steps in vec(step(), 1..MAX_STEPS),
        keyboard in select(SnipKeyboard::ALL.to_vec()),
        countdown_s in 0..=MAX_COUNT_S,
        mic_on in any::<bool>(),
    ) {
        let mut world = World::new(Rig::with(|s| *s = settings_of(keyboard, countdown_s, mic_on)));
        for step in steps.iter().flat_map(plain) {
            one(&mut world, &step)?;
        }
        check_escs(&mut world)?;
        world.act(&Step::Quit)?;
        prop_assert_eq!(world.rig.stage(), &Stage::Idle);
        world.check_idle()?;
    }
}
