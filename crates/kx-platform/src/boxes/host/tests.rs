//! The vault job over a scripted worker: each call sends its vault ask with the next id and maps the answer to the
//! port's result. Faults, refusals and a late, dead or lying worker have their own files.

mod faults;
mod life;

use super::*;
use crate::boxes::times;
use crate::worker::fake::{Call, Hands, Script, VaultAct};
use crate::worker::{Host, Launch};
use kx_ipc::{
    BoxPart as Part, Dropped, KeepassEntry as Entry, KeepassField as Field, NOTEBOOK_PART,
    VAULT_JOB, VAULT_MEMORY_MB, VaultAsk, VaultReply, Wiped,
};
use kx_module_api::ports::{BoxConfig, KeepassEntry, KeepassField, MIN_BOX_WORK};
use kx_module_api::{Redacted, Secret, Zeroizing};
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

/// Test values; the real ones come from the vault module's settings.
const CFG: BoxConfig = BoxConfig {
    deadline_ms: 20_000,
    idle_s: 30,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};

/// A test password that appears nowhere else, in Arabic ("password").
const PASSWORD: &str = "كلمة السر";
/// A test note that appears nowhere else, in Arabic ("my secret").
const NOTE: &str = "سري الخاص";
/// The work factor the tests ask, the lowest the worker takes.
const WORK: u8 = MIN_BOX_WORK;
/// A test padlock's open half.
const RECIPIENT: &str = "age1recipient";
/// A test padlock's locked identity.
const LOCKED: &[u8] = b"locked identity";
/// A test box's bytes; the host reads none of them.
const BOX: &[u8] = b"a sealed box";
/// A test `KeePass` file's bytes.
const KDBX: &[u8] = b"a kdbx file";

/// The host the tests drive.
type Vault = Host<Script<VaultFrames>, VaultFrames>;

fn host(script: &Script<VaultFrames>, clock: &Hands) -> Vault {
    let (spawner, clock) = (Arc::new(script.clone()), Arc::new(clock.clone()));
    Host::new(spawner, clock, times(&CFG), Launch::vault())
}

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

fn padlock() -> Padlock {
    Padlock {
        recipient: Redacted::new(RECIPIENT.to_owned()),
        locked: Redacted::new(LOCKED.to_vec()),
    }
}

/// A part named `name` holding the note.
fn part(name: &str) -> BoxPart {
    let bytes = Redacted::new(Zeroizing::new(NOTE.as_bytes().to_vec()));
    let name = name.to_owned();
    BoxPart { name, bytes }
}

/// The notebook part as a frame carries it.
fn notebook() -> Part<'static> {
    let bytes = Wiped::Lent(NOTE.as_bytes());
    let name = Cow::Borrowed(NOTEBOOK_PART);
    Part { name, bytes }
}

/// The test password as a frame carries it.
fn wiped() -> Wiped<'static> {
    Wiped::Lent(PASSWORD.as_bytes())
}

/// Every vault ask the workers got, in order.
fn asks(script: &Script<VaultFrames>) -> Vec<VaultAsk<'static>> {
    let vault = |call| match call {
        Call::Vault(ask) => Some(ask),
        _ => None,
    };
    script.calls().into_iter().filter_map(vault).collect()
}

/// One call of each kind, as the module makes them, with its answer dropped.
#[derive(Copy, Clone, Debug)]
enum Kind {
    Padlock,
    Seal,
    Open,
    Check,
    Keepass,
}

const KINDS: [Kind; 5] = [
    Kind::Padlock,
    Kind::Seal,
    Kind::Open,
    Kind::Check,
    Kind::Keepass,
];

fn call(host: &mut Vault, kind: Kind) -> Result<(), BoxError> {
    match kind {
        Kind::Padlock => host.padlock(secret(PASSWORD), WORK).map(drop),
        Kind::Seal => host.seal(padlock(), vec![part(NOTEBOOK_PART)]).map(drop),
        Kind::Open => host.open(BOX.to_vec(), secret(PASSWORD)).map(drop),
        Kind::Check => host.check(padlock(), secret(PASSWORD)).map(drop),
        Kind::Keepass => host.keepass(KDBX.to_vec(), secret(PASSWORD)).map(drop),
    }
}

fn padlocked(id: u32) -> VaultReply {
    let (recipient, locked) = (RECIPIENT.to_owned(), LOCKED.to_vec());
    VaultReply::Padlock {
        id,
        recipient,
        locked,
    }
}

fn sealed(id: u32) -> VaultReply {
    let bytes = BOX.to_vec();
    VaultReply::Sealed { id, bytes }
}

fn opened(id: u32) -> VaultReply {
    let bytes = Wiped::Kept(Zeroizing::new(NOTE.as_bytes().to_vec()));
    let parts = vec![Part {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes,
    }];
    let (recipient, locked) = (RECIPIENT.to_owned(), LOCKED.to_vec());
    VaultReply::Opened {
        id,
        recipient,
        locked,
        parts,
    }
}

fn checked(id: u32) -> VaultReply {
    VaultReply::Checked { id, yes: false }
}

fn entries(id: u32) -> VaultReply {
    let text = |t: &str| Zeroizing::new(t.to_owned());
    let custom = vec![Field {
        label: text("PIN"),
        value: text(NOTE),
    }];
    let entry = Entry {
        title: text("بنك"),
        password: text(NOTE),
        custom,
        ..Entry::default()
    };
    let dropped = Dropped {
        entries: 2,
        fields: 3,
    };
    VaultReply::Entries {
        id,
        entries: vec![entry],
        dropped,
    }
}

#[test]
fn each_call_sends_its_vault_ask_with_the_next_id() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    for (kind, answer) in KINDS
        .into_iter()
        .zip([padlocked, sealed, opened, checked, entries])
    {
        script.act(VaultAct::Answer(answer));
        assert_eq!(call(&mut host, kind), Ok(()), "{kind:?}");
    }
    assert_eq!(asks(&script), sent());
    assert_eq!(script.spawns(), 1, "one worker answers each in turn");
}

/// The asks the five calls send, in turn.
fn sent() -> [VaultAsk<'static>; 5] {
    let (recipient, locked) = (Cow::Borrowed(RECIPIENT), Cow::Borrowed(LOCKED));
    [
        VaultAsk::Padlock {
            id: 1,
            work: WORK,
            password: wiped(),
        },
        VaultAsk::Seal {
            id: 2,
            recipient,
            locked: locked.clone(),
            parts: vec![notebook()],
        },
        VaultAsk::Open {
            id: 3,
            password: wiped(),
            bytes: Cow::Borrowed(BOX),
        },
        VaultAsk::Check {
            id: 4,
            password: wiped(),
            locked,
        },
        VaultAsk::Keepass {
            id: 5,
            password: wiped(),
            bytes: Cow::Borrowed(KDBX),
        },
    ]
}

#[test]
fn each_answer_becomes_the_ports_result() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let acts = [padlocked, sealed, opened, checked, entries].map(VaultAct::Answer);
    for act in acts {
        script.act(act);
    }
    assert_eq!(host.padlock(secret(PASSWORD), WORK), Ok(padlock()));
    let sealed = host.seal(padlock(), vec![part(NOTEBOOK_PART)]);
    assert_eq!(sealed.map(Redacted::into_inner), Ok(BOX.to_vec()));
    let parts = vec![part(NOTEBOOK_PART)];
    let opened = Opened {
        padlock: padlock(),
        parts,
    };
    assert_eq!(host.open(BOX.to_vec(), secret(PASSWORD)), Ok(opened));
    assert_eq!(host.check(padlock(), secret(PASSWORD)), Ok(false));
    let pin = KeepassField {
        label: Redacted::new("PIN".to_owned()),
        value: secret(NOTE),
    };
    let entry = KeepassEntry {
        title: secret("بنك"),
        user: secret(""),
        password: secret(NOTE),
        address: secret(""),
        notes: secret(""),
        custom: vec![pin],
    };
    let dropped = Dropped {
        entries: 2,
        fields: 3,
    };
    let import = KeepassImport {
        entries: vec![entry],
        dropped,
    };
    assert_eq!(host.keepass(KDBX.to_vec(), secret(PASSWORD)), Ok(import));
}

#[test]
fn the_vault_job_starts_alone_with_its_own_memory_cap() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(VaultAct::Answer(checked));
    call(&mut host, Kind::Check).unwrap();
    let args = vec![VAULT_JOB.to_owned()];
    let memory_mb = VAULT_MEMORY_MB;
    assert_eq!(script.launches(), [Launch { args, memory_mb }]);
    let within = Duration::from_millis(CFG.deadline_ms.into());
    assert!(
        script.calls().ends_with(&[Call::Reply { within }]),
        "within vault_ms"
    );
}
