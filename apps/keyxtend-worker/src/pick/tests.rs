//! The pick job over scripted dialogs: each ask shows its dialog with the asked values, a path is told only once it
//! passes the ask's check and is refused otherwise, a Cancel is told, a dialog that did not show tells its code, a
//! panic is a failure, and the job ends on goodbye or the end of input.

use super::*;
use kx_ipc::{FileKind, MAX_PATH_BYTES};
use kx_platform_record::pick::Unshown;
use kx_test_support::paths::native;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The folder a test dialog opens at, and the name it proposes.
const SNIPS: &str = r"D:\KeyXtend\snips";
const NAME: &str = "Snip 2026-11-24 10-30-05.png";
/// A file a PNG Save may answer, in Arabic: "Documents", "my snips", "screenshot".
const CHOSEN: &str = r"D:\المستندات\لقطاتي\لقطة الشاشة.png";
/// A folder the folder dialog may answer.
const FOLDER: &str = r"E:\Pictures";
/// Answers that are no plain absolute path or of another type than PNG, which no PNG Save tells.
const REFUSED: [&str; 6] = [
    r"\\server\share\snip.png",
    r"\\?\D:\snip.png",
    r"snips\snip.png",
    r"D:\snips\..\snip.png",
    r"D:\snips\snip.jpg",
    r"D:\snips\snip",
];
/// A system code a dialog that did not show keeps: `E_ACCESSDENIED`.
const DENIED: u32 = 0x8007_0005;

/// One dialog as the job showed it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Shown {
    Save(String, String, FileKind),
    Folder(String),
}

/// What a scripted dialog does: answer, not show, or panic.
#[derive(Clone, Debug)]
enum Act {
    Answer(Result<Picked, Unshown>),
    Panic,
}

/// Dialogs that play their acts in turn and keep each dialog shown.
#[derive(Default)]
struct Script {
    acts: RefCell<VecDeque<Act>>,
    shown: RefCell<Vec<Shown>>,
}

impl Script {
    fn with(acts: impl IntoIterator<Item = Act>) -> Self {
        Self {
            acts: RefCell::new(acts.into_iter().collect()),
            shown: RefCell::default(),
        }
    }

    /// Notes `shown` and plays the next act.
    fn play(&self, shown: Shown) -> Result<Picked, Unshown> {
        self.shown.borrow_mut().push(shown);
        match self
            .acts
            .borrow_mut()
            .pop_front()
            .expect("an act for each dialog")
        {
            Act::Answer(answer) => answer,
            Act::Panic => panic!("the test dialog panics"),
        }
    }
}

impl Dialogs for Script {
    fn save(&self, folder: &str, name: &str, kind: FileKind) -> Result<Picked, Unshown> {
        self.play(Shown::Save(folder.to_owned(), name.to_owned(), kind))
    }

    fn folder(&self, at: &str) -> Result<Picked, Unshown> {
        self.play(Shown::Folder(at.to_owned()))
    }
}

/// An answer of `path`, written for Windows and read in this system's style.
fn path(path: &str) -> Act {
    Act::Answer(Ok(Picked::Path(native(path))))
}

/// A PNG Save ask `id` at `SNIPS`, proposing `NAME`.
fn save(id: u32) -> PickAsk<'static> {
    PickAsk::Save {
        id,
        folder: Cow::Borrowed(SNIPS),
        name: Cow::Borrowed(NAME),
        kind: FileKind::Png,
    }
}

/// A folder ask `id` at `SNIPS`.
fn folder(id: u32) -> PickAsk<'static> {
    PickAsk::Folder {
        id,
        at: Cow::Borrowed(SNIPS),
    }
}

/// Runs the job over `script` with `asks` on its input, and hands back how it ended and every answer.
fn run_over(
    script: &Script,
    asks: &[PickAsk<'_>],
) -> (Result<(), IpcError>, Vec<PickReply<'static>>) {
    let mut input = Vec::new();
    for ask in asks {
        ask.write(&mut input).unwrap();
    }
    let mut out = Vec::new();
    let ended = run(&mut &input[..], &mut out, script);
    let mut told = &out[..];
    let replies = std::iter::from_fn(|| PickReply::read(&mut told).ok()).collect();
    (ended, replies)
}

#[test]
fn a_save_shows_its_dialog_as_asked_and_tells_the_file_chosen() {
    let script = Script::with([path(CHOSEN)]);
    let (ended, replies) = run_over(&script, &[save(1)]);
    assert_eq!(ended, Ok(()));
    let picked = PickReply::Picked {
        id: 1,
        path: Cow::Owned(native(CHOSEN)),
    };
    assert_eq!(replies, [picked]);
    let asked = Shown::Save(SNIPS.to_owned(), NAME.to_owned(), FileKind::Png);
    assert_eq!(*script.shown.borrow(), [asked]);
}

#[test]
fn a_folder_ask_shows_the_folder_dialog_and_tells_the_folder_chosen() {
    let script = Script::with([path(FOLDER)]);
    let (_, replies) = run_over(&script, &[folder(2)]);
    let picked = PickReply::Picked {
        id: 2,
        path: Cow::Owned(native(FOLDER)),
    };
    assert_eq!(replies, [picked]);
    assert_eq!(*script.shown.borrow(), [Shown::Folder(SNIPS.to_owned())]);
}

#[test]
fn a_cancel_is_told_and_a_dialog_that_did_not_show_tells_its_code() {
    let script = Script::with([
        Act::Answer(Ok(Picked::Cancelled)),
        Act::Answer(Err(Unshown { code: DENIED })),
    ]);
    let (_, replies) = run_over(&script, &[save(1), folder(2)]);
    let failed = PickReply::Failed {
        id: 2,
        code: DENIED,
    };
    assert_eq!(replies, [PickReply::Cancelled { id: 1 }, failed]);
}

#[test]
fn an_answer_of_no_plain_absolute_path_or_of_another_type_is_refused_never_told() {
    let script = Script::with(REFUSED.map(path));
    let ids = 1..=u32::try_from(REFUSED.len()).unwrap();
    let (_, replies) = run_over(&script, &ids.clone().map(save).collect::<Vec<_>>());
    let refused: Vec<_> = ids.map(|id| PickReply::Refused { id }).collect();
    assert_eq!(replies, refused);
    let script = Script::with([path(r"\\server\share"), path(CHOSEN)]);
    let (_, replies) = run_over(&script, &[folder(7), save(8)]);
    assert_eq!(replies[0], PickReply::Refused { id: 7 });
    assert!(
        matches!(replies[1], PickReply::Picked { id: 8, .. }),
        "the job goes on"
    );
}

#[test]
fn an_answer_too_long_for_the_reply_is_refused_and_the_job_goes_on() {
    let long = format!(r"D:\{}", vec!["d"; MAX_PATH_BYTES].join(r"\"));
    let script = Script::with([path(&long), path(FOLDER)]);
    let (ended, replies) = run_over(&script, &[folder(1), folder(2)]);
    assert_eq!(ended, Ok(()));
    assert_eq!(replies[0], PickReply::Refused { id: 1 });
    assert!(matches!(replies[1], PickReply::Picked { id: 2, .. }));
}

#[test]
fn a_dialog_that_panics_is_a_failure_with_no_code_and_the_job_goes_on() {
    let script = Script::with([Act::Panic, path(CHOSEN)]);
    let (ended, replies) = run_over(&script, &[save(1), save(2)]);
    assert_eq!(ended, Ok(()));
    let failed = PickReply::Failed {
        id: 1,
        code: PickReply::NO_CODE,
    };
    assert_eq!(replies[0], failed);
    assert!(matches!(replies[1], PickReply::Picked { id: 2, .. }));
}

#[test]
fn goodbye_ends_the_job_before_any_later_ask_and_a_broken_frame_ends_it_with_its_error() {
    let script = Script::with([path(CHOSEN)]);
    let (ended, replies) = run_over(&script, &[save(1), PickAsk::Bye, save(2)]);
    assert_eq!((ended, replies.len()), (Ok(()), 1));
    let mut broken = &b"not a frame at all"[..];
    let ended = run(&mut broken, &mut Vec::new(), &Script::default());
    assert_eq!(ended, Err(IpcError::BadMagic));
    assert!(Script::default().shown.borrow().is_empty());
}
