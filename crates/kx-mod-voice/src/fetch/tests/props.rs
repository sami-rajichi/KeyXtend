//! Any run of starts, button presses and notes over two rows, as the panel starts one only while none is live: one
//! download at a time, a button acts only where its row shows it, a note of another row is stale and changes nothing,
//! and every end leaves no download live.

use super::*;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

/// The rows a case downloads.
const IDS: [&str; 2] = ["parakeet-v3-q8", "whisper-small-q5km"];
/// The most steps in one case.
const MAX_STEPS: usize = 40;

/// The faults a download may end on.
const FAULTS: [ModelError; 14] = [
    ModelError::Print,
    ModelError::Size,
    ModelError::Full,
    ModelError::Missing,
    ModelError::ReadOnly,
    ModelError::BadPath,
    ModelError::Io,
    ModelError::Net,
    ModelError::Stalled,
    ModelError::Host,
    ModelError::Server,
    ModelError::Internal,
    ModelError::Gone,
    ModelError::Cancelled,
];

/// One thing the downloads meet, each on a row of `IDS`.
#[derive(Clone, Debug)]
enum Step {
    /// Start the row's download on a drive with room when true.
    Start(usize, bool),
    Press(usize, FetchButton),
    /// Progress of this many bytes of this total.
    Progress(usize, u64, u64),
    Done(usize),
    Fault(usize, usize),
}

fn step() -> impl Strategy<Value = Step> {
    let at = || 0..IDS.len();
    prop_oneof![
        2 => (at(), any::<bool>()).prop_map(|(i, room)| Step::Start(i, room)),
        3 => (at(), select(FetchButton::ALL.to_vec())).prop_map(|(i, b)| Step::Press(i, b)),
        3 => (at(), any::<u64>(), any::<u64>()).prop_map(|(i, d, t)| Step::Progress(i, d, t)),
        1 => at().prop_map(Step::Done),
        1 => (at(), 0..FAULTS.len()).prop_map(|(i, f)| Step::Fault(i, f)),
    ]
}

fn row(i: usize) -> &'static Row {
    table().unwrap().get(IDS[i]).unwrap()
}

/// The live download as the case expects it: its row and where it stands.
type Want = Option<(usize, FetchState)>;

/// Starts row `i`'s download when none is live: it starts at 0 when the drive has room, else says Full.
fn start(
    fetches: &mut Fetches,
    live: &mut Want,
    i: usize,
    room: bool,
) -> Result<(), TestCaseError> {
    if live.is_some() {
        return Ok(());
    }
    let (r, s) = (row(i), settings());
    let free = r.bytes + u64::from(s.min_free_mb) * MB;
    let drives = drive(if room { free } else { free - 1 });
    let got = fetches.start(r, &Folder::new(&dir()), &drives, &s);
    if room {
        prop_assert_eq!(got.map(|ask| ask.bytes), Ok(r.bytes));
        let state = FetchState {
            done: 0,
            total: r.bytes,
            paused: false,
        };
        *live = Some((i, state));
    } else {
        prop_assert_eq!(got.map(|ask| ask.bytes), Err(BarHint::Full));
    }
    Ok(())
}

/// Presses `button` on row `i`: it acts only on the live row, Pause only while it runs and Resume only while paused.
fn press(
    fetches: &mut Fetches,
    live: &mut Want,
    i: usize,
    button: FetchButton,
) -> Result<(), TestCaseError> {
    let got = fetches.press(IDS[i], button);
    let want = match live.as_mut().filter(|(at, _)| *at == i) {
        Some((_, state)) => match button {
            FetchButton::Pause if !state.paused => {
                state.paused = true;
                Some(button)
            }
            FetchButton::Resume if state.paused => {
                state.paused = false;
                Some(button)
            }
            FetchButton::Cancel => Some(button),
            FetchButton::Pause | FetchButton::Resume => None,
        },
        None => None,
    };
    if want == Some(FetchButton::Cancel) {
        *live = None;
    }
    prop_assert_eq!(got, want);
    Ok(())
}

/// Hands row `i` `note`: another row's is stale; progress moves it, and an end leaves none live.
fn note(
    fetches: &mut Fetches,
    live: &mut Want,
    i: usize,
    note: FetchNote,
) -> Result<(), TestCaseError> {
    let got = fetches.note(IDS[i], note);
    let Some((_, state)) = live.as_mut().filter(|(at, _)| *at == i) else {
        prop_assert_eq!(got, Fetched::Stale);
        return Ok(());
    };
    let want = match note {
        FetchNote::Progress { done, total } => {
            (state.done, state.total) = (done, total);
            Fetched::Moved
        }
        FetchNote::Done => Fetched::Done,
        FetchNote::Fault(e) => hint(e).map_or(Fetched::Stopped, Fetched::Failed),
    };
    if want != Fetched::Moved {
        *live = None;
    }
    prop_assert_eq!(got, want);
    Ok(())
}

proptest! {
    #[test]
    fn one_download_at_a_time_and_each_button_where_its_row_shows_it(steps in vec(step(), 1..MAX_STEPS)) {
        let (mut fetches, mut live) = (Fetches::default(), None);
        for step in &steps {
            match *step {
                Step::Start(i, room) => start(&mut fetches, &mut live, i, room)?,
                Step::Press(i, button) => press(&mut fetches, &mut live, i, button)?,
                Step::Progress(i, done, total) => {
                    note(&mut fetches, &mut live, i, FetchNote::Progress { done, total })?;
                }
                Step::Done(i) => note(&mut fetches, &mut live, i, FetchNote::Done)?,
                Step::Fault(i, f) => note(&mut fetches, &mut live, i, FetchNote::Fault(FAULTS[f]))?,
            }
            prop_assert_eq!(fetches.busy(), live.is_some());
            for (i, id) in IDS.iter().enumerate() {
                let want = live.filter(|(at, _)| *at == i).map(|(_, s)| s);
                prop_assert_eq!(fetches.state(id), want, "row {}", id);
            }
        }
    }
}
