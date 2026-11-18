//! Voice typing's events and ports (ADR-0042, ADR-0046): the voice events travel only with `Voice`, the level with no
//! grant, the speech port needs `Microphone`, the model files port `Network`, the folder reveal `OpenFolder`, and the
//! session watch nothing.

use kx_module_api::Capability::{
    Microphone, Network, OpenFolder, ReadFocusedText, SecretStore, Voice, WriteClipboard,
};
use kx_module_api::ports::{
    BarPlace, FolderError, FolderPath, FolderRevealKey, JobLimits, ModelFilesKey, PanelKey,
    PanelLine, SessionWatchKey, SpeechAsk, SpeechError, SpeechKey, TextAsk, TextDone, VoiceAct,
    VoiceAsk, VoiceBar, VoiceClick, VoiceClose, VoiceKey, VoiceLevel, VoiceLine, VoiceLineKey,
    VoiceLit, VoiceOpen, VoicePage, VoiceView,
};
use kx_module_api::{
    Bus, Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, Redacted, ServiceError,
    ServiceKey, Zeroizing,
};
use kx_test_support::host::FakeHost;
use std::sync::Arc;
use stub::Stub;

mod stub;

/// A voice module declaring its powers (ADR-0042 decisions 7 and 8, ADR-0046).
static VOICE: Manifest = Manifest {
    id: ModuleId::new("voice"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[
        SpeechKey::ID,
        ModelFilesKey::ID,
        SessionWatchKey::ID,
        FolderRevealKey::ID,
    ],
    provides: &[],
    capabilities: &[
        Voice,
        Microphone,
        Network,
        SecretStore,
        ReadFocusedText,
        WriteClipboard,
        OpenFolder,
    ],
    settings: None,
};

/// The module's powers besides `Voice`, none of which opens a voice event.
const OTHERS: &[Capability] = &[
    Microphone,
    Network,
    SecretStore,
    ReadFocusedText,
    WriteClipboard,
    OpenFolder,
];

/// Puts `api` into `host` under `K`'s id and gate.
fn put<K: ServiceKey>(host: &mut FakeHost, api: Arc<K::Api>) {
    host.services.insert(K::ID, (K::CAPABILITY, Box::new(api)));
}

/// A host granted `granted`, holding the three ports as the platform would provide them.
fn host(granted: &[Capability]) -> FakeHost {
    let mut host = FakeHost::new(&VOICE);
    host.granted.extend(granted);
    put::<SpeechKey>(&mut host, Arc::new(Stub));
    put::<ModelFilesKey>(&mut host, Arc::new(Stub));
    put::<SessionWatchKey>(&mut host, Arc::new(Stub));
    put::<FolderRevealKey>(&mut host, Arc::new(Stub));
    host
}

/// An empty view.
fn page() -> VoicePage {
    VoicePage {
        round: 1,
        bar: VoiceBar::Hidden,
        panel: None,
        place: BarPlace::Above,
        sounds: true,
        cloud: None,
    }
}

/// A text ask for "hello".
fn ask() -> TextAsk {
    let text = Redacted::new(Zeroizing::new("hello".to_owned()));
    TextAsk {
        round: 2,
        erase: 0,
        text,
        gap_ms: 10,
    }
}

/// Publishes one of each gated voice event on `bus`.
fn sends(bus: &Bus) -> [Result<(), EventError>; 11] {
    let act = VoiceAct {
        round: 1,
        click: VoiceClick::Over(true),
    };
    [
        bus.publish(&VoiceKey),
        bus.publish(&VoiceLit::Prep),
        bus.publish(&VoiceOpen),
        bus.publish(&VoiceAsk),
        bus.publish(&VoiceView::new(page())),
        bus.publish(&act),
        bus.publish(&ask()),
        bus.publish(&TextDone {
            round: 2,
            count: 5,
            ok: true,
        }),
        bus.publish(&VoiceLineKey(PanelKey::Enter)),
        bus.publish(&VoiceLine(Some(PanelLine::Private))),
        bus.publish(&VoiceClose),
    ]
}

/// The refusal of `E` for want of `Voice`.
fn refused<E: Event>() -> Result<(), EventError> {
    Err(EventError::NotGranted(E::NAME, Voice))
}

#[test]
fn without_voice_no_voice_event_travels() {
    let want = [
        refused::<VoiceKey>(),
        refused::<VoiceLit>(),
        refused::<VoiceOpen>(),
        refused::<VoiceAsk>(),
        refused::<VoiceView>(),
        refused::<VoiceAct>(),
        refused::<TextAsk>(),
        refused::<TextDone>(),
        refused::<VoiceLineKey>(),
        refused::<VoiceLine>(),
        refused::<VoiceClose>(),
    ];
    assert_eq!(sends(&host(OTHERS).bus()), want);
    let bus = host(OTHERS).bus();
    assert_eq!(
        bus.subscribe(|_: &TextAsk| {}).err(),
        Some(EventError::NotGranted(TextAsk::NAME, Voice))
    );
}

#[test]
fn with_voice_a_text_ask_reaches_its_listener_whole() {
    let bus = host(&[Voice]).bus();
    assert_eq!(sends(&bus), [Ok(()); 11]);
    let (tx, rx) = std::sync::mpsc::channel();
    let _sub = bus
        .subscribe(move |a: &TextAsk| tx.send(a.clone()).unwrap())
        .unwrap();
    bus.publish(&ask()).unwrap();
    let got: Vec<_> = rx.try_iter().collect();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].text.expose().as_str(), "hello");
}

#[test]
fn the_level_passes_with_no_grant() {
    let bus = host(&[]).bus();
    assert_eq!(bus.publish(&VoiceLevel { level: 40 }), Ok(()));
    assert!(bus.subscribe(|_: &VoiceLevel| {}).is_ok());
}

#[test]
fn speech_needs_the_microphone_and_model_files_the_network() {
    let mut host = host(&[Voice, SecretStore]);
    let cx = ModuleCx::new(&mut host);
    let speech = ServiceError::NotGranted(SpeechKey::ID, Microphone);
    assert_eq!(cx.service::<SpeechKey>().err(), Some(speech));
    let files = ServiceError::NotGranted(ModelFilesKey::ID, Network);
    assert_eq!(cx.service::<ModelFilesKey>().err(), Some(files));
}

#[test]
fn the_folder_reveal_needs_open_folder() {
    let mut bare = host(&[Voice, Microphone, Network]);
    let cx = ModuleCx::new(&mut bare);
    let refused = ServiceError::NotGranted(FolderRevealKey::ID, OpenFolder);
    assert_eq!(cx.service::<FolderRevealKey>().err(), Some(refused));
    let mut granted = host(&[OpenFolder]);
    let reveal = ModuleCx::new(&mut granted)
        .service::<FolderRevealKey>()
        .unwrap();
    let folder = FolderPath::new(r"D:\KeyXtend\models".into()).unwrap();
    assert_eq!(reveal.show(&folder), Err(FolderError::Missing));
}

#[test]
fn the_speech_port_names_the_check_clips_languages() {
    let mut host = host(&[Microphone]);
    let speech = ModuleCx::new(&mut host).service::<SpeechKey>().unwrap();
    assert_eq!(
        speech.clips(),
        ["en", "fr", "ar"],
        "as the host staged them"
    );
}

#[test]
fn the_session_watch_needs_no_grant() {
    let mut host = host(&[]);
    let watch = ModuleCx::new(&mut host).service::<SessionWatchKey>();
    assert!(watch.is_ok(), "a lock or a sleep carries no content");
}

/// A listen-only ask: no model, so the job records and hands its sound over at Stop.
fn listen() -> SpeechAsk {
    let job = JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    };
    let (model, card) = (None, None);
    SpeechAsk {
        job,
        model,
        card,
        threads: 0,
        piece_s: 30,
        pad_ms: 300,
        level_hz: 30,
        record_max_s: 600,
        silence_stop_s: None,
    }
}

#[test]
fn a_listen_only_ask_passes_and_speech_is_reached_through_its_gate() {
    assert_eq!(
        listen().check(),
        Ok(()),
        "no model: the job records and hands its sound over"
    );
    let over = SpeechAsk {
        level_hz: 0,
        ..listen()
    };
    assert_eq!(over.check(), Err(SpeechError::Range("level_hz")));
    let mut host = host(&[Microphone]);
    assert!(ModuleCx::new(&mut host).service::<SpeechKey>().is_ok());
}
