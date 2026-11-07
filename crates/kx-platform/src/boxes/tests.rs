//! Sessions on their own threads over a scripted vault worker: every call goes through, a dropped session stops its
//! worker, an idle worker stops by itself, and a worker that never starts fails each call.

use super::*;
use crate::worker::fake::{Call, Hands, Script, VaultAct};
use kx_ipc::{BoxPart as Part, Dropped, NOTEBOOK_PART, VaultReply, Wiped};
use kx_module_api::Zeroizing;
use kx_module_api::ports::MIN_BOX_WORK;
use std::borrow::Cow;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// How long a test waits for the session thread; a wait, never a sleep.
const WAIT: Duration = Duration::from_secs(10);

/// Test values; the real ones come from the vault module's settings.
const CFG: BoxConfig = BoxConfig {
    deadline_ms: 20_000,
    idle_s: 30,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};

/// A test password that appears nowhere else.
const PASSWORD: &str = "zebra-quartz";

/// The scripted vault worker the sessions start.
type Vault = Script<VaultFrames>;

/// Boxes over `script`, and a channel that hears each call to its workers.
fn boxes(script: &Vault) -> (WorkerBoxes<Vault>, Receiver<Call>) {
    let (tell, told) = mpsc::channel();
    script.tell(tell);
    let boxes = WorkerBoxes::new(script.clone(), Arc::new(Hands::default()));
    (boxes, told)
}

/// Waits until the session's worker is ended.
fn ended(told: &Receiver<Call>) {
    while told.recv_timeout(WAIT).expect("the worker is ended") != Call::End {}
}

fn password() -> Secret {
    Redacted::new(Zeroizing::new(PASSWORD.to_owned()))
}

fn padlock() -> Padlock {
    Padlock {
        recipient: Redacted::new("age1".to_owned()),
        locked: Redacted::new(vec![1]),
    }
}

fn padlocked(id: u32) -> VaultReply {
    let (recipient, locked) = ("age1".to_owned(), vec![1]);
    VaultReply::Padlock {
        id,
        recipient,
        locked,
    }
}

fn opened(id: u32) -> VaultReply {
    let bytes = Wiped::Kept(Zeroizing::new(vec![2]));
    let parts = vec![Part {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes,
    }];
    let (recipient, locked) = ("age1".to_owned(), vec![1]);
    VaultReply::Opened {
        id,
        recipient,
        locked,
        parts,
    }
}

#[test]
fn every_call_goes_through_the_session_thread_and_its_drop_stops_the_worker() {
    let script = Script::default();
    let (boxes, told) = boxes(&script);
    let mut run = boxes.start(CFG);
    let sealed = |id| VaultReply::Sealed { id, bytes: vec![3] };
    let checked = |id| VaultReply::Checked { id, yes: true };
    let entries = |id| VaultReply::Entries {
        id,
        entries: Vec::new(),
        dropped: Dropped::default(),
    };
    for answer in [padlocked, sealed, opened, checked, entries] {
        script.act(VaultAct::Answer(answer));
    }
    assert_eq!(run.padlock(&password(), MIN_BOX_WORK), Ok(padlock()));
    let note = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes: Redacted::new(Zeroizing::new(vec![2])),
    };
    let sealed = run.seal(&padlock(), std::slice::from_ref(&note));
    assert_eq!(sealed.map(Redacted::into_inner), Ok(vec![3]));
    let opened = run.open(&[3], &password()).map(|o| o.parts);
    assert_eq!(opened, Ok(vec![note]));
    assert_eq!(run.check(&padlock(), &password()), Ok(true));
    assert_eq!(run.keepass(&[4], &password()), Ok(KeepassImport::default()));
    assert_eq!(script.spawns(), 1);
    drop(run);
    ended(&told);
    let within = Duration::from_millis(CFG.deadline_ms.into());
    assert!(
        script
            .calls()
            .ends_with(&[Call::Bye, Call::Code { within }, Call::End])
    );
}

#[test]
fn a_session_stops_an_idle_worker_by_itself() {
    let script = Script::default();
    let (boxes, told) = boxes(&script);
    let mut run = boxes.start(BoxConfig { idle_s: 0, ..CFG });
    script.act(VaultAct::Answer(padlocked));
    run.padlock(&password(), MIN_BOX_WORK).unwrap();
    ended(&told);
    assert!(script.calls().contains(&Call::Bye));
}

#[test]
fn a_worker_that_never_starts_fails_each_call() {
    let script = Script::default();
    script.refuse(usize::MAX);
    let (boxes, _told) = boxes(&script);
    let mut run = boxes.start(CFG);
    let note = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes: Redacted::new(Zeroizing::new(vec![2])),
    };
    let calls = [
        run.padlock(&password(), MIN_BOX_WORK).map(drop),
        run.seal(&padlock(), &[note]).map(drop),
        run.open(&[3], &password()).map(drop),
        run.check(&padlock(), &password()).map(drop),
        run.keepass(&[4], &password()).map(drop),
    ];
    assert_eq!(calls, [Err(BoxError::Failed); 5]);
    assert_eq!(script.spawns(), 0);
    assert_eq!(script.tries(), 1, "no start within the backoff");
}
