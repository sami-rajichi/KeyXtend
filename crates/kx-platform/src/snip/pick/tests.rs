//! The dialogs host over a scripted pick worker: each dialog in the pick job with the foreground granted first and
//! held to its limits, the answer checked again, the job's refusals as the port's errors, and no short deadline.

use super::*;
use crate::UNSAVED_DIR;
use crate::worker::fake::{Call, Hands, PickAct, Script};
use kx_ipc::{FileKind as Kind, IpcError, PickReply};
use kx_platform_fake::FakeSnipFolder;
use std::path::PathBuf;
use std::time::Duration;

mod limits;
mod save;

/// Each pick limit at the start of its range.
const fn limits() -> PickLimits {
    PickLimits {
        job_mb: *PickLimits::JOB_MB.start(),
        dialog_min: *PickLimits::DIALOG_MIN.start(),
        idle_s: *PickLimits::IDLE_S.start(),
        restart_ms: *PickLimits::RESTART_MS.start(),
        restart_max_ms: *PickLimits::RESTART_MAX_MS.start(),
    }
}

/// How long a dialog held to `limits` waits on the user.
fn dialog(limits: &PickLimits) -> Duration {
    Duration::from_mins(u64::from(limits.dialog_min))
}

/// An answer the scripted job gives, made for its dialog's id.
type Answers = fn(u32) -> PickReply<'static>;

/// The Snips folder, under a neutral user's folder.
const SNIPS: &str = "C:/Users/someone/KeyXtend/snips";
/// Where the user saves, and the name proposed.
const PICTURES: &str = "C:/Users/someone/Pictures";
/// The proposed name.
const NAME: &str = "Snip 1.png";
/// The file the user chose.
const CHOSEN: &str = r"C:\Users\someone\Pictures\Shot.png";

/// A dialogs host over `script`, with the fake Snips folder.
fn asker(script: &Script<PickFrames>) -> WorkerFileAsk<Script<PickFrames>> {
    let clock: Arc<dyn Clock> = Arc::new(Hands::default());
    let (snips, unsaved) = (PathBuf::from(SNIPS), PathBuf::from(SNIPS).join(UNSAVED_DIR));
    let (snips, unsaved) = (
        FolderPath::new(snips).unwrap(),
        FolderPath::new(unsaved).unwrap(),
    );
    let folder = FakeSnipFolder::new(Arc::clone(&clock), snips, unsaved);
    WorkerFileAsk::new(script.clone(), clock, Arc::new(folder))
}

/// The picture proposed in the user's Pictures folder.
fn proposed() -> SnipFile {
    SnipFile::new(PathBuf::from(PICTURES).join(NAME), FileKind::Png).unwrap()
}

/// The user chose `CHOSEN`.
fn chosen(id: u32) -> PickReply<'static> {
    PickReply::Picked {
        id,
        path: CHOSEN.into(),
    }
}

/// The dialogs the pick job was asked for, in order.
fn dialogs(script: &Script<PickFrames>) -> Vec<PickAsk<'static>> {
    let calls = script.calls().into_iter();
    calls
        .filter_map(|c| match c {
            Call::Pick(ask) => Some(ask),
            _ => None,
        })
        .collect()
}

#[test]
fn a_save_shows_the_dialog_in_the_pick_job_with_the_foreground_granted_first() {
    let script = Script::default();
    let asker = asker(&script);
    script.act(PickAct::Answer(chosen));
    let saved = asker
        .save(&proposed(), &limits())
        .unwrap()
        .expect("a file chosen");
    let want = SnipFile::new(PathBuf::from(CHOSEN), FileKind::Png).unwrap();
    assert_eq!(saved.file(), &want);
    assert_eq!(script.launches(), [Launch::pick(limits().job_mb)]);
    let shown = PickAsk::Save {
        id: 1,
        folder: PICTURES.into(),
        name: NAME.into(),
        kind: Kind::Png,
    };
    assert_eq!(dialogs(&script), [shown]);
    let calls = script.calls();
    let front = calls.iter().position(|c| *c == Call::Front);
    let pick = calls.iter().position(|c| matches!(c, Call::Pick(_)));
    assert!(
        front.is_some() && front < pick,
        "granted before the dialog: {calls:?}"
    );
}

#[test]
fn the_jobs_answers_become_the_ports_with_one_worker() {
    let script = Script::default();
    let asker = asker(&script);
    let answers: [(Answers, Option<AskError>); 3] = [
        (|id| PickReply::Cancelled { id }, None),
        (|id| PickReply::Refused { id }, Some(AskError::BadPath)),
        (
            |id| PickReply::Failed { id, code: 5 },
            Some(AskError::Failed),
        ),
    ];
    for (reply, want) in answers {
        script.act(PickAct::Answer(reply));
        let got = asker
            .save(&proposed(), &limits())
            .map(|a| a.map(|s| s.file().clone()));
        assert_eq!(got.err(), want);
    }
    assert_eq!(script.spawns(), 1, "one worker for every answer");
}

#[test]
fn an_answer_its_own_check_should_have_refused_ends_the_worker() {
    let wrong = [r"C:\Users\someone\Shot.mp4", r"\\server\share\Shot.png"];
    for path in wrong {
        let script = Script::default();
        let asker = asker(&script);
        script.act(PickAct::Path(path.to_owned()));
        assert_eq!(
            asker.save(&proposed(), &limits()).err(),
            Some(AskError::Gone)
        );
        let calls = script.calls();
        assert!(
            calls.contains(&Call::End) && !calls.contains(&Call::Bye),
            "{calls:?}"
        );
    }
}

/// The unsaved part's path, as the dialog answers it, with `rest` after it.
fn in_unsaved(rest: &str) -> String {
    format!(r"{}\{UNSAVED_DIR}{rest}", SNIPS.replace('/', r"\"))
}

#[test]
fn a_save_or_folder_in_the_unsaved_part_is_refused_and_the_worker_kept() {
    let script = Script::default();
    let asker = asker(&script);
    let shot = in_unsaved(r"\Shot.png");
    for answer in [shot.clone(), shot.to_uppercase()] {
        script.act(PickAct::Path(answer));
        assert_eq!(
            asker.save(&proposed(), &limits()).err(),
            Some(AskError::BadPath)
        );
    }
    for answer in [in_unsaved(""), in_unsaved(r"\deeper")] {
        script.act(PickAct::Path(answer));
        assert_eq!(
            asker.folder(&pictures(), &limits()).err(),
            Some(AskError::BadPath)
        );
    }
    // The Snips folder itself, where the dialog opens, still takes a snip.
    script.act(PickAct::Path(format!(
        r"{}\Shot.png",
        SNIPS.replace('/', r"\")
    )));
    assert!(asker.save(&proposed(), &limits()).unwrap().is_some());
    assert_eq!(script.spawns(), 1, "the worker kept");
}

#[test]
fn a_dialog_waits_its_minutes_and_a_silent_broken_or_missing_worker_is_gone() {
    let (silent, broken, missing) = (Script::default(), Script::default(), Script::default());
    silent.act(PickAct::Silent);
    assert_eq!(
        asker(&silent).save(&proposed(), &limits()).err(),
        Some(AskError::Gone)
    );
    let within = dialog(&limits());
    assert!(silent.calls().contains(&Call::Reply { within }));
    broken.act(PickAct::Broken(IpcError::BadField));
    assert_eq!(
        asker(&broken).folder(&pictures(), &limits()).err(),
        Some(AskError::Gone)
    );
    missing.refuse(1);
    assert_eq!(
        asker(&missing).save(&proposed(), &limits()).err(),
        Some(AskError::Gone)
    );
    assert_eq!(missing.spawns(), 0);
}

#[test]
fn a_name_the_frames_refuse_shows_no_dialog() {
    let script = Script::default();
    let asker = asker(&script);
    let odd = SnipFile::new(PathBuf::from(PICTURES).join("a?.png"), FileKind::Png).unwrap();
    assert_eq!(asker.save(&odd, &limits()).err(), Some(AskError::Failed));
    assert_eq!(script.tries(), 0);
}

#[test]
fn the_folder_dialog_answers_a_folder_or_nothing() {
    let script = Script::default();
    let asker = asker(&script);
    script.act(PickAct::Answer(|id| PickReply::Picked {
        id,
        path: r"D:\Shots".into(),
    }));
    let want = FolderPath::new(PathBuf::from(r"D:\Shots")).unwrap();
    assert_eq!(asker.folder(&pictures(), &limits()), Ok(Some(want)));
    script.act(PickAct::Answer(|id| PickReply::Cancelled { id }));
    assert_eq!(asker.folder(&pictures(), &limits()), Ok(None));
    let at = PickAsk::Folder {
        id: 1,
        at: PICTURES.into(),
    };
    assert_eq!(dialogs(&script)[0], at);
}

/// The user's Pictures folder.
fn pictures() -> FolderPath {
    FolderPath::new(PathBuf::from(PICTURES)).unwrap()
}
