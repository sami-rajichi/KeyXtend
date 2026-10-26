//! Word prediction's events and ports (ADR-0033): typed text reaches only a module holding `ReadFocusedText`, the
//! field watch and the sealed store sit behind their gates, and the packs and the press request and ask are open to
//! all.

use kx_module_api::Capability::{InjectInput, ReadFocusedText, SecretStore};
use kx_module_api::ports::{
    BlobName, ChipPicked, FieldChange, FieldConfig, FieldEnds, FieldError, FieldKind, FieldRead,
    FieldRun, FieldWatch, FieldWatchKey, Lane, PackError, PackView, Piece, PressesAsk,
    PressesWanted, SealedError, SealedStore, SealedStoreKey, Suggested, TypeWord, Typed,
    TypingMode, TypingModeAsk, WordPacks, WordPacksKey, WordsAsk,
};
use kx_module_api::{
    Bus, Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, Redacted, ServiceError,
    ServiceKey,
};
use kx_test_support::host::FakeHost;
use std::sync::{Arc, Mutex};

/// A prediction module declaring both gates, and `InjectInput` to show that another grant opens neither.
static PREDICT: Manifest = Manifest {
    id: ModuleId::new("predict"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[FieldWatchKey::ID, WordPacksKey::ID, SealedStoreKey::ID],
    provides: &[],
    capabilities: &[ReadFocusedText, SecretStore, InjectInput],
    settings: None,
};

/// What the one box holds before its cursor ("I am here").
const TEXT: &str = "أنا هنا";
/// How that text ends ("here").
const END: &str = "هنا";
/// The one box's id.
const BOX: u64 = 9;

/// A watch on one text box, which it tells at once.
struct OneBox;

/// The run over that box.
struct OneRun;

impl FieldWatch for OneBox {
    fn watch(
        &self,
        _: FieldConfig,
        tell: Box<dyn Fn(FieldChange) + Send>,
    ) -> Result<Box<dyn FieldRun>, FieldError> {
        tell(FieldChange {
            kind: FieldKind::Text,
            box_id: BOX,
        });
        Ok(Box::new(OneRun))
    }
}

impl FieldRun for OneRun {
    fn kind(&self) -> FieldKind {
        FieldKind::Text
    }

    fn before_cursor(&mut self, _: u8) -> Result<FieldRead, FieldError> {
        let text = Some(Redacted::new(TEXT.to_owned()));
        Ok(FieldRead {
            kind: FieldKind::Text,
            text,
        })
    }

    fn ends_with(&mut self, end: &Redacted<String>) -> FieldEnds {
        FieldEnds {
            kind: FieldKind::Text,
            ends: Some(TEXT.ends_with(end.expose().as_str())),
        }
    }
}

/// Packs for no language.
struct NoPacks;

impl WordPacks for NoPacks {
    fn open(&self, _: &str) -> Result<Option<Box<dyn PackView>>, PackError> {
        Ok(None)
    }

    fn pack_of(&self, _: &str) -> Option<String> {
        None
    }
}

/// A store whose OS key is unavailable.
struct Locked;

impl SealedStore for Locked {
    fn load(&self, _: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        Err(SealedError::Locked)
    }

    fn save(&self, _: &BlobName, _: &[u8]) -> Result<(), SealedError> {
        Err(SealedError::Locked)
    }
}

/// A host granted `granted`, holding the three ports as the platform would provide them.
fn host(granted: &[Capability]) -> FakeHost {
    let field: Arc<dyn FieldWatch> = Arc::new(OneBox);
    let packs: Arc<dyn WordPacks> = Arc::new(NoPacks);
    let sealed: Arc<dyn SealedStore> = Arc::new(Locked);
    let mut host = FakeHost::new(&PREDICT);
    host.granted.extend(granted);
    let services = &mut host.services;
    services.insert(
        FieldWatchKey::ID,
        (FieldWatchKey::CAPABILITY, Box::new(field)),
    );
    services.insert(
        WordPacksKey::ID,
        (WordPacksKey::CAPABILITY, Box::new(packs)),
    );
    services.insert(
        SealedStoreKey::ID,
        (SealedStoreKey::CAPABILITY, Box::new(sealed)),
    );
    host
}

/// Sends one of each gated event on `bus`.
fn send_all(bus: &Bus) -> [Result<(), EventError>; 7] {
    let lane = Suggested {
        round: 1,
        lane: Lane::Empty,
        forgetting: false,
        learn: true,
        on: true,
        list: None,
    };
    let mode = TypingMode {
        shift: true,
        ..TypingMode::default()
    };
    let word = TypeWord {
        round: 1,
        erase: 0,
        text: Redacted::default(),
    };
    [
        bus.publish(&Typed {
            piece: Piece::Erase,
        }),
        bus.publish(&mode),
        bus.publish(&TypingModeAsk),
        bus.publish(&word),
        bus.publish(&lane),
        bus.publish(&ChipPicked { round: 1, index: 0 }),
        bus.publish(&WordsAsk::Again),
    ]
}

/// Subscribes to each gated event on `bus`, in `send_all`'s order, and drops each subscription at once.
fn hear_all(bus: &Bus) -> [Result<(), EventError>; 7] {
    [
        bus.subscribe(|_: &Typed| {}).map(drop),
        bus.subscribe(|_: &TypingMode| {}).map(drop),
        bus.subscribe(|_: &TypingModeAsk| {}).map(drop),
        bus.subscribe(|_: &TypeWord| {}).map(drop),
        bus.subscribe(|_: &Suggested| {}).map(drop),
        bus.subscribe(|_: &ChipPicked| {}).map(drop),
        bus.subscribe(|_: &WordsAsk| {}).map(drop),
    ]
}

/// The refusal of `E` for want of `ReadFocusedText`.
fn refused<E: Event>() -> Result<(), EventError> {
    Err(EventError::NotGranted(E::NAME, ReadFocusedText))
}

#[test]
fn without_read_focused_text_no_typed_text_travels() {
    let bus = host(&[InjectInput, SecretStore]).bus();
    let want = [
        refused::<Typed>(),
        refused::<TypingMode>(),
        refused::<TypingModeAsk>(),
        refused::<TypeWord>(),
        refused::<Suggested>(),
        refused::<ChipPicked>(),
        refused::<WordsAsk>(),
    ];
    assert_eq!(send_all(&bus), want);
    assert_eq!(hear_all(&bus), want, "no one may listen either");
}

#[test]
fn with_read_focused_text_a_typed_piece_reaches_its_listener_whole() {
    let bus = host(&[ReadFocusedText]).bus();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let _sub = bus
        .subscribe(move |t: &Typed| log.lock().unwrap().push(t.piece.clone()))
        .unwrap();
    let piece = Piece::Text(Redacted::new(TEXT.to_owned()));
    bus.publish(&Typed {
        piece: piece.clone(),
    })
    .unwrap();
    assert_eq!(*seen.lock().unwrap(), [piece]);
    let all = [Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(())];
    assert_eq!(send_all(&bus), all);
    assert_eq!(hear_all(&bus), all);
}

#[test]
fn a_press_request_and_its_ask_pass_on_a_bus_with_no_grants() {
    let bus = host(&[]).bus();
    assert_eq!(bus.publish(&PressesWanted { on: true }), Ok(()));
    assert_eq!(bus.publish(&PressesAsk), Ok(()));
}

#[test]
fn the_field_watch_and_the_sealed_store_need_their_gates() {
    let mut host = host(&[InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let field = ServiceError::NotGranted(FieldWatchKey::ID, ReadFocusedText);
    assert_eq!(cx.service::<FieldWatchKey>().err(), Some(field));
    let sealed = ServiceError::NotGranted(SealedStoreKey::ID, SecretStore);
    assert_eq!(cx.service::<SealedStoreKey>().err(), Some(sealed));
    assert!(
        cx.service::<WordPacksKey>().is_ok(),
        "the packs need no grant"
    );
}

#[test]
fn a_watch_reached_through_its_gate_tells_the_box_and_reads_before_the_cursor() {
    let mut host = host(&[ReadFocusedText]);
    let watch = ModuleCx::new(&mut host).service::<FieldWatchKey>().unwrap();
    let told = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&told);
    let tell = Box::new(move |c: FieldChange| log.lock().unwrap().push(c));
    let config = FieldConfig {
        read_ms: 1,
        read_chars: 1,
    };
    let mut run = watch.watch(config, tell).unwrap();
    let change = FieldChange {
        kind: FieldKind::Text,
        box_id: BOX,
    };
    assert_eq!(*told.lock().unwrap(), [change]);
    assert_eq!(run.kind(), FieldKind::Text);
    let read = run.before_cursor(2).unwrap();
    assert_eq!(read.kind, FieldKind::Text);
    assert_eq!(read.text.map(Redacted::into_inner).as_deref(), Some(TEXT));
    let ended = run.ends_with(&Redacted::new(END.to_owned()));
    assert_eq!((ended.kind, ended.ends), (FieldKind::Text, Some(true)));
}

#[test]
fn the_store_and_the_packs_reached_through_their_gates_answer() {
    let mut host = host(&[SecretStore]);
    let cx = ModuleCx::new(&mut host);
    let name = BlobName::new("predict-en").unwrap();
    let store = cx.service::<SealedStoreKey>().unwrap();
    assert_eq!(store.load(&name), Err(SealedError::Locked));
    assert_eq!(store.save(&name, &[]), Err(SealedError::Locked));
    let packs = cx.service::<WordPacksKey>().unwrap();
    assert!(packs.open("en").unwrap().is_none());
}
