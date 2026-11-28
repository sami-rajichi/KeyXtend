//! Snip's events and ports (ADR-0056): the events travel only with the `Snip` gate, which reaches no port, as voice
//! typing's do (ADR-0042); the freeze, the recorder and the Snips folder need `ScreenCapture`, and the microphone's
//! pass `Microphone`, each a kernel check. A test named `example_` shows a port's contract on the stand-in `Stub`, the
//! adapters' tests prove it (Tasks 4, 7 and 8), and the dialogs, playing and a kept copy are in `files`.

use kx_module_api::Capability::{
    Microphone, OpenFolder, SaveFile, ScreenCapture, Snip, WriteClipboard,
};
use kx_module_api::ports::{
    ClipWait, ClipboardWriterKey, FileAskKey, FileKind, FreezeError, MediaOpenKey, MicPass,
    MicPassKey, MicPasses, MicSeal, PanelKey, PanelLine, Part, PicturesKey, Point, RecordAsk,
    RecordDone, RecordEnd, RecordError, RecordNote, RecorderKey, Rect, SHOT_CAP_MB, Screen,
    ScreenFreezeKey, ShotCap, SnipAct, SnipAside, SnipAsk, SnipClick, SnipFolderKey, SnipLine,
    SnipLineKey, SnipLit, SnipPage, SnipStage, SnipStart, SnipView,
};
use kx_module_api::{
    Bus, Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, ServiceError,
    ServiceKey,
};
use kx_test_support::host::FakeHost;
use std::sync::{Arc, Mutex};
use stub::{DESKTOP_MB, LENGTH_MS, SEAL, Stub, waits};

mod files;
mod stub;

/// A snip module declaring its powers (ADR-0056 decision 10).
static SNIP: Manifest = Manifest {
    id: ModuleId::new("snip"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[
        ScreenFreezeKey::ID,
        RecorderKey::ID,
        SnipFolderKey::ID,
        MicPassKey::ID,
        FileAskKey::ID,
        MediaOpenKey::ID,
        ClipboardWriterKey::ID,
        PicturesKey::ID,
    ],
    provides: &[],
    capabilities: &[
        Snip,
        ScreenCapture,
        Microphone,
        WriteClipboard,
        OpenFolder,
        SaveFile,
    ],
    settings: None,
};

/// The module's powers besides the `Snip` gate, none of which opens a snip event.
const OTHERS: &[Capability] = &[
    ScreenCapture,
    Microphone,
    WriteClipboard,
    OpenFolder,
    SaveFile,
];
/// A clipboard wait that gives up at once.
const NOW: ClipWait = ClipWait {
    open_ms: 0,
    poll_ms: 0,
};

/// Puts `api` into `host` under `K`'s id and gate.
fn put<K: ServiceKey>(host: &mut FakeHost, api: Arc<K::Api>) {
    host.services.insert(K::ID, (K::CAPABILITY, Box::new(api)));
}

/// A host granted `granted`, holding the ports as the platform and the app would provide them.
fn host(granted: &[Capability]) -> FakeHost {
    let mut host = FakeHost::new(&SNIP);
    host.granted.extend(granted);
    put::<ScreenFreezeKey>(&mut host, Arc::new(Stub));
    put::<RecorderKey>(&mut host, Arc::new(Stub));
    put::<SnipFolderKey>(&mut host, Arc::new(Stub));
    put::<MicPassKey>(&mut host, Arc::new(SEAL.clone()));
    put::<FileAskKey>(&mut host, Arc::new(Stub));
    put::<MediaOpenKey>(&mut host, Arc::new(Stub));
    put::<ClipboardWriterKey>(&mut host, Arc::new(Stub));
    put::<PicturesKey>(&mut host, Arc::new(Stub));
    host
}

/// Fails unless `cx` is refused `K` for want of `cap`.
fn refuses<K: ServiceKey>(cx: &ModuleCx<'_>, cap: Capability) {
    let want = ServiceError::NotGranted(K::ID, cap);
    assert_eq!(cx.service::<K>().err(), Some(want), "{}", K::ID.as_str());
}

/// A page with nothing to draw.
fn idle() -> SnipPage {
    SnipPage {
        round: 1,
        frozen: Vec::new(),
        stage: SnipStage::Idle,
        ask: None,
        note: None,
        panel: None,
        dialog: false,
    }
}

/// Publishes one of each snip event on `bus`.
fn sends(bus: &Bus) -> [Result<(), EventError>; 8] {
    let act = SnipAct {
        round: 1,
        click: SnipClick::Spot(Point { x: 5, y: 7 }),
    };
    [
        bus.publish(&SnipStart),
        bus.publish(&SnipView::new(idle())),
        bus.publish(&act),
        bus.publish(&SnipAside::Bubble),
        bus.publish(&SnipLit::On),
        bus.publish(&SnipLine(Some(PanelLine::None))),
        bus.publish(&SnipLineKey(PanelKey::Esc)),
        bus.publish(&SnipAsk),
    ]
}

/// The refusal of `E` for want of the `Snip` gate.
fn refused<E: Event>() -> Result<(), EventError> {
    Err(EventError::NotGranted(E::NAME, Snip))
}

#[test]
fn without_the_snip_gate_no_snip_event_travels() {
    let want = [
        refused::<SnipStart>(),
        refused::<SnipView>(),
        refused::<SnipAct>(),
        refused::<SnipAside>(),
        refused::<SnipLit>(),
        refused::<SnipLine>(),
        refused::<SnipLineKey>(),
        refused::<SnipAsk>(),
    ];
    let bus = host(OTHERS).bus();
    assert_eq!(sends(&bus), want);
    let listen = bus.subscribe(|_: &SnipAct| {}).err();
    assert_eq!(listen, Some(EventError::NotGranted(SnipAct::NAME, Snip)));
}

#[test]
fn with_the_snip_gate_a_click_reaches_its_listener_whole() {
    let bus = host(&[Snip]).bus();
    assert_eq!(sends(&bus), [Ok(()); 8]);
    let (tx, rx) = std::sync::mpsc::channel();
    let _sub = bus
        .subscribe(move |a: &SnipAct| tx.send(a.clone()).unwrap())
        .unwrap();
    let act = SnipAct {
        round: 3,
        click: SnipClick::Spot(Point { x: -1920, y: 40 }),
    };
    let work = Rect {
        x: -1920,
        y: 0,
        w: 1920,
        h: 1040,
    };
    let areas = SnipAct {
        round: 3,
        click: SnipClick::Screens(vec![Screen {
            rect: work,
            dpi: 96,
            primary: false,
        }]),
    };
    let answers = [SnipClick::Aside, SnipClick::Bare].map(|click| SnipAct { round: 3, click });
    bus.publish(&act).unwrap();
    bus.publish(&areas).unwrap();
    for answer in &answers {
        bus.publish(answer).unwrap();
    }
    let [aside, bare] = answers;
    assert_eq!(rx.try_iter().collect::<Vec<_>>(), [act, areas, aside, bare]);
}

#[test]
fn the_snip_gate_alone_reaches_no_port_as_a_face_holds_it() {
    let mut face = host(&[Snip]);
    let cx = ModuleCx::new(&mut face);
    refuses::<ScreenFreezeKey>(&cx, ScreenCapture);
    refuses::<RecorderKey>(&cx, ScreenCapture);
    refuses::<SnipFolderKey>(&cx, ScreenCapture);
    refuses::<MicPassKey>(&cx, Microphone);
}

/// Screen capture reaches the freeze and the folder; on the stand-in, a desktop past the cap freezes nothing.
#[test]
fn example_screen_capture_reaches_the_freeze_and_the_folder() {
    let mut granted = host(&[ScreenCapture]);
    let cx = ModuleCx::new(&mut granted);
    let freeze = cx.service::<ScreenFreezeKey>().unwrap();
    let least = ShotCap::new(*SHOT_CAP_MB.start()).unwrap();
    assert!(least.mb() < DESKTOP_MB, "the stub's desktop is bigger");
    assert_eq!(
        freeze.freeze(least).err(),
        Some(FreezeError::TooBig),
        "past the cap"
    );
    let most = ShotCap::new(*SHOT_CAP_MB.end()).unwrap();
    assert_eq!(freeze.freeze(most).map(|f| f.screens.len()), Ok(1));
    let folder = cx.service::<SnipFolderKey>().unwrap();
    assert!(folder.snips().is_some() && folder.free().is_ok());
}

/// A 2 by 2 ask at the first rate with the computer's sound and the microphone when `mic` says, each value at the
/// start of its range.
#[allow(clippy::unwrap_used, reason = "a 2 by 2 part is never empty")]
fn record_ask(mic: Option<MicPass>) -> RecordAsk {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 2,
        h: 2,
    };
    RecordAsk {
        part: Part::new(rect).unwrap(),
        fps: RecordAsk::FPS[0],
        mic,
        sound: true,
        max_min: *RecordAsk::MAX_MIN.start(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits: waits(),
    }
}

/// Only `Microphone` reaches the app's seal; on the stand-in, as every recorder must, a pass from a module's own seal
/// is refused.
#[test]
fn example_a_recording_hears_the_microphone_only_with_the_pass_microphone_reaches() {
    let mut no_mic = host(&[ScreenCapture]);
    refuses::<MicPassKey>(&ModuleCx::new(&mut no_mic), Microphone);
    let mut granted = host(&[ScreenCapture, Microphone]);
    let cx = ModuleCx::new(&mut granted);
    let pass = cx.service::<MicPassKey>().unwrap().pass();
    let recorder = cx.service::<RecorderKey>().unwrap();
    let minted = Some(MicSeal::new().pass());
    let refused = recorder.start(record_ask(minted.clone()), Box::new(|_| {}));
    assert_eq!(
        refused.err(),
        Some(RecordError::BadPass),
        "a module's own seal"
    );
    let mut run = recorder
        .start(record_ask(Some(pass.clone())), Box::new(|_| {}))
        .unwrap();
    assert_eq!((run.mic(None), run.mic(Some(pass))), (Ok(()), Ok(())));
    assert_eq!(
        run.mic(minted),
        Err(RecordError::BadPass),
        "a module's own pass"
    );
}

/// On the stand-in: a recording checks its ask, then tells its time and its end.
#[test]
fn example_a_recording_checks_its_ask_and_tells_its_time_then_its_end() {
    let mut granted = host(&[ScreenCapture]);
    let recorder = ModuleCx::new(&mut granted)
        .service::<RecorderKey>()
        .unwrap();
    let bad = RecordAsk {
        fps: (0..=u8::MAX).find(|f| !RecordAsk::FPS.contains(f)).unwrap(),
        ..record_ask(None)
    };
    let refused = recorder.start(bad, Box::new(|_| {})).err();
    assert_eq!(refused, Some(RecordError::Range("fps")));
    let notes = Arc::new(Mutex::new(Vec::new()));
    let heard = Arc::clone(&notes);
    let tell = Box::new(move |n| heard.lock().unwrap().push(n));
    let run = recorder.start(record_ask(None), tell).unwrap();
    let done = RecordDone {
        length_ms: LENGTH_MS,
        end: RecordEnd::Stopped,
    };
    let told = [RecordNote::Time { ms: 0 }, RecordNote::Done(done)];
    assert_eq!(*notes.lock().unwrap(), told, "it started, then it ended");
    assert_eq!(run.file().kind(), FileKind::Mp4);
}
