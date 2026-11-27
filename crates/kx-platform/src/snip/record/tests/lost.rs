//! How a run ends badly or early: a lost job is one fault and deletes the broken video (edge 13), and a new ask starts
//! a new job; a fault mid-run ends it the same way, a disk filling up too; a cancel tells nothing more and deletes the
//! video even after Stop; a dropped run says goodbye and keeps its video for the sweep.

use super::*;
use kx_module_api::ports::UnsavedFile;
use std::thread;

/// Why a start failed whose job died once it was asked.
fn quit_at_start(rig: &Rig) -> Option<RecordError> {
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    let recorder = Arc::clone(&rig.recorder);
    let starting = thread::spawn(move || recorder.start(ask(), ear().0).err());
    assert!(matches!(sent(&asked), Ask::Start { .. }));
    rig.script.quit(DIED);
    starting.join().unwrap()
}

#[test]
fn a_job_lost_before_it_started_is_gone_and_its_video_goes_if_made() {
    let rig = Rig::new();
    assert_eq!(quit_at_start(&rig), Some(RecordError::Gone));
    assert_eq!(rig.folder.removes().len(), 1);
    rig.folder.refuse_removes(SaveError::Io, 1);
    assert_eq!(quit_at_start(&rig), Some(RecordError::Gone));
    let removed = rig.folder.removes();
    assert_eq!(removed.len(), 3, "held, then tried again: {removed:?}");
}

#[test]
fn a_job_lost_mid_run_is_one_fault_deletes_the_broken_video_and_a_new_ask_starts_a_new_job() {
    let rig = Rig::new();
    let (mut run, heard) = rig.live();
    rig.script.quit(DIED);
    assert_eq!(next(&heard), RecordNote::Fault(RecordError::Gone));
    assert_eq!(rig.folder.removes(), [run.file().clone()], "edge 13");
    assert_eq!(run.stop(), Err(RecordError::Gone));
    assert_eq!(run.mic(None), Err(RecordError::Gone));
    assert!(heard.try_recv().is_err(), "one fault, then nothing");
    let (again, _heard) = rig.live();
    assert_ne!(again.file(), run.file());
    assert_eq!(rig.script.spawns(), 2, "a new job");
}

#[test]
fn a_fault_mid_run_ends_it_and_deletes_its_video_a_full_drive_too() {
    let rig = Rig::new();
    for (code, error) in [
        (RecordFault::Failed, RecordError::Failed),
        (RecordFault::Full, RecordError::Full),
    ] {
        let (run, heard) = rig.live();
        rig.script.say(Ok(fault(1, code)));
        assert_eq!(next(&heard), RecordNote::Fault(error));
        assert!(rig.folder.removes().contains(run.file()));
        rig.close(run);
    }
}

#[test]
fn a_cancel_tells_nothing_more_and_deletes_the_video_even_when_the_job_could_not() {
    let rig = Rig::new();
    let (mut run, heard) = rig.live();
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    run.cancel();
    assert_eq!(sent(&asked), Ask::Cancel { id: 1 }, "posted");
    rig.script.say(Ok(failed(1)));
    rig.script.say(Ok(second(1)));
    assert!(
        rig.folder.wait_removes(1, LONG),
        "the job could not, so the host does"
    );
    assert_eq!(rig.folder.removes(), [run.file().clone()]);
    drop(run);
    assert!(
        heard.recv_timeout(LONG).is_err(),
        "nothing told after a cancel"
    );
}

#[test]
fn a_cancel_from_inside_its_own_callback_still_reaches_the_job() {
    let rig = Rig::new();
    let slot: Arc<Mutex<Option<Box<dyn RecordRun>>>> = Arc::default();
    let inner = Arc::clone(&slot);
    let tell = Box::new(move |note: RecordNote| {
        let mut held = lock(&inner);
        if let (RecordNote::Time { ms: 1.. }, Some(run)) = (note, held.as_mut()) {
            run.cancel();
        }
    });
    rig.script.act(RecordAct::Say(vec![started]));
    let run = rig.recorder.start(ask(), tell).unwrap();
    *lock(&slot) = Some(run);
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    rig.script.say(Ok(second(1)));
    assert_eq!(sent(&asked), Ask::Cancel { id: 1 }, "posted, not asked");
    let run = lock(&slot).take();
    drop(run);
}

#[test]
fn a_cancel_after_stop_but_before_the_end_deletes_the_video_at_its_end() {
    let rig = Rig::new();
    let (mut run, heard) = rig.live();
    assert_eq!(run.stop(), Ok(()));
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    run.cancel();
    assert_eq!(sent(&asked), Ask::Cancel { id: 1 }, "posted");
    rig.script.say(Ok(stopped(1)));
    assert!(
        rig.folder.wait_removes(1, LONG),
        "its end kept it, so the host deletes it"
    );
    assert_eq!(rig.folder.removes(), [run.file().clone()]);
    drop(run);
    assert!(heard.recv_timeout(LONG).is_err(), "its end is not told");
}

/// The script's calls as they stood at each remove, oldest first.
type Seen = Arc<Mutex<Vec<Vec<Call>>>>;

/// The fake folder, keeping the script's calls as they stood at each remove.
struct AtRemove {
    folder: FakeSnipFolder,
    script: Script<RecordFrames>,
    seen: Seen,
}

impl SnipFolder for AtRemove {
    fn snips(&self) -> Option<FolderPath> {
        self.folder.snips()
    }

    fn unsaved(&self) -> Result<Vec<UnsavedFile>, SaveError> {
        self.folder.unsaved()
    }

    fn mark_copied(&self, video: &SnipFile) -> Result<(), SaveError> {
        self.folder.mark_copied(video)
    }

    fn prune_marks(&self) -> Result<(), SaveError> {
        self.folder.prune_marks()
    }

    fn remove(&self, file: &SnipFile) -> Result<(), SaveError> {
        lock(&self.seen).push(self.script.calls());
        self.folder.remove(file)
    }

    fn free(&self) -> Result<u64, SaveError> {
        self.folder.free()
    }
}

#[test]
fn a_cancelled_run_dropped_before_its_job_answers_is_deleted_once_its_job_quit() {
    let seen = Seen::default();
    let rig = Rig::with(|folder, script| {
        let (folder, script, seen) = (folder.clone(), script.clone(), Arc::clone(&seen));
        Arc::new(AtRemove {
            folder,
            script,
            seen,
        })
    });
    let (mut run, _heard) = rig.live();
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    run.cancel();
    assert_eq!(sent(&asked), Ask::Cancel { id: 1 });
    let file = run.file().clone();
    drop(run);
    assert!(rig.folder.wait_removes(1, LONG));
    assert_eq!(rig.folder.removes(), [file]);
    let at = lock(&seen).first().cloned().unwrap_or_default();
    let within = ms(ask().waits.end_ms);
    let ended = [Call::Bye, Call::Code { within }, Call::End];
    assert!(at.ends_with(&ended), "goodbye, then the job's end: {at:?}");
}

#[test]
fn a_broken_video_the_folder_still_held_goes_once_the_run_is_left() {
    let rig = Rig::new();
    rig.folder.refuse_removes(SaveError::Io, 1);
    let (run, heard) = rig.live();
    let file = run.file().clone();
    rig.script.quit(DIED);
    assert_eq!(next(&heard), RecordNote::Fault(RecordError::Gone));
    drop(run);
    let again = rig.folder.wait_removes(2, LONG);
    assert!(again, "tried again once its job ended");
    assert_eq!(rig.folder.removes(), [file.clone(), file]);
}

#[test]
fn a_dropped_run_says_goodbye_and_keeps_its_video_for_the_sweep() {
    let rig = Rig::new();
    let (run, _heard) = rig.live();
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    drop(run);
    assert_eq!(asked.recv_timeout(LONG), Ok(Call::Bye));
    assert!(rig.folder.removes().is_empty());
}
