//! The job over the real engine and a fake library: pieces written in the model's language while listening goes on,
//! and engine asks answered by notes, or by `Runner` without an engine. Notes are awaited, never slept for.

use super::*;
use crate::voice::cut::NoDetectors;
use crate::voice::engine::NoEngine;
use crate::voice::fake::lib::{FakeLib, device};
use crate::voice::library::Library;
use crate::voice::run::Local;
use kx_ipc::DeviceKind;
use std::time::Duration;

/// The longest the tests wait for one event, far past what the fake engine needs.
const WAIT: Duration = Duration::from_secs(30);

impl Rig {
    /// Takes events as the engine's thread sends them until `seen` or the notes since hold one `last` picks.
    fn until(
        &mut self,
        last: impl Fn(&VoiceNote) -> bool,
        mut seen: Vec<VoiceNote>,
    ) -> Vec<VoiceNote> {
        while !seen.iter().chain(&self.notes).any(&last) {
            let event = self
                .rx
                .recv_timeout(WAIT)
                .expect("an event within the wait");
            let flow = self.job.take(event, &mut self.notes).unwrap();
            assert!(matches!(flow, Flow::Go));
        }
        seen.extend(self.told());
        seen
    }
}

/// The test program itself, an existing file to name as a model.
fn exe() -> String {
    std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

/// A job over the fake source, a detector of loud frames and the engine over a fake library that knows `fr-FR`.
fn engine_rig() -> (Rig, FakeLib) {
    let lib = FakeLib::default();
    lib.script().langs = vec!["fr-FR".into()];
    let mic = fake::mic(RATE, 1);
    let parts = Parts {
        source: Box::new(fake::FakeSource(Rc::clone(&mic))),
        detectors: Box::new(fake::FakeDetectors(true)),
        engine: Box::new(Local::new(lib.clone(), 8)),
    };
    (Rig::over(mic, Rc::default(), parts), lib)
}

/// Run 1 on the fake library's model, in French on the processor.
fn writing() -> VoiceAsk<'static> {
    let mut setup = setup(false);
    setup.model = Some(Model {
        path: Cow::Owned(exe()),
        lang: Cow::Borrowed("fr-FR"),
        runner: Cow::Borrowed(FakeLib::RUNNER),
        target: Target::Cpu,
        threads: 0,
    });
    VoiceAsk::Start { id: 1, setup }
}

/// The piece note the fake library tells for `samples` in French.
fn said(index: u16, samples: usize) -> VoiceNote {
    let text = zeroize::Zeroizing::new(format!("{samples} fr-FR"));
    VoiceNote::Piece { id: 1, index, text }
}

#[test]
fn the_engine_writes_pieces_in_the_models_language_while_listening_goes_on() {
    let (mut rig, lib) = engine_rig();
    let mut heard = rig.ask(writing());
    let mut talk = loud(12 * SEC);
    talk.extend(silent(SEC));
    talk.extend(loud(3 * SEC));
    heard.extend(rig.speak(&talk, CHUNK));
    let notes = rig.until(|n| matches!(n, VoiceNote::Piece { .. }), heard);
    assert!(rig.open(), "listening goes on");
    let want = [
        stage(1, Stage::Open),
        stage(1, Stage::Loading),
        said(0, 12 * SEC + PAD),
    ];
    assert_eq!(plain(&notes), want);
    rig.only(VoiceAsk::Stop { id: 1 });
    let mut notes = rig.until(|n| matches!(n, VoiceNote::Done { .. }), Vec::new());
    let Some(VoiceNote::Done {
        pieces,
        sound_ms,
        card,
        ..
    }) = notes.pop()
    else {
        panic!("no done note: {notes:?}");
    };
    assert_eq!((pieces, sound_ms, card), (2, 16_000, false));
    assert_eq!(
        plain(&notes),
        [stage(1, Stage::Shut), said(1, PAD + 3 * SEC)]
    );
    assert!(lib.script().runs.iter().all(|r| r.2 == "fr-FR"));
}

#[test]
fn sound_heard_while_the_model_still_loads_is_all_written_once_it_loads() {
    let (mut rig, lib) = engine_rig();
    let (go, gate) = mpsc::channel();
    lib.script().gate = Some(gate);
    let heard = rig.ask(writing());
    let notes = rig.until(|n| *n == stage(1, Stage::Loading), heard);
    assert_eq!(
        plain(&notes),
        [stage(1, Stage::Open), stage(1, Stage::Loading)]
    );
    let mut talk = loud(12 * SEC);
    talk.extend(silent(SEC));
    talk.extend(loud(3 * SEC));
    let heard = rig.speak(&talk, CHUNK);
    assert!(
        lib.script().runs.is_empty(),
        "nothing is written while it loads"
    );
    go.send(()).unwrap();
    rig.only(VoiceAsk::Stop { id: 1 });
    let notes = rig.until(|n| matches!(n, VoiceNote::Done { .. }), heard);
    let pieces: Vec<VoiceNote> = (notes.iter())
        .filter(|n| matches!(n, VoiceNote::Piece { .. }))
        .cloned()
        .collect();
    assert_eq!(pieces, [said(0, 12 * SEC + PAD), said(1, PAD + 3 * SEC)]);
}

#[test]
fn the_engines_answers_reach_the_host_and_free_their_ask() {
    let (mut rig, lib) = engine_rig();
    lib.script().found = vec![device(DeviceKind::Cpu, "CPU", 8000)];
    let is = |id| move |n: &VoiceNote| n.id() == id;
    rig.only(VoiceAsk::Devices { id: 3 });
    let notes = rig.until(is(3), Vec::new());
    assert!(
        matches!(notes[..], [VoiceNote::Devices { id: 3, .. }]),
        "{notes:?}"
    );
    for _ in 0..2 {
        let path = Cow::Owned(exe());
        rig.only(VoiceAsk::Facts { id: 4, path });
        let notes = rig.until(is(4), Vec::new());
        assert!(
            matches!(notes[..], [VoiceNote::Facts { id: 4, .. }]),
            "{notes:?}"
        );
    }
}

#[test]
fn without_an_engine_every_engine_ask_answers_runner() {
    let mic = fake::mic(RATE, 1);
    let parts = Parts {
        source: Box::new(fake::FakeSource(Rc::clone(&mic))),
        detectors: Box::new(NoDetectors),
        engine: Box::new(NoEngine),
    };
    let mut rig = Rig::over(mic, Rc::default(), parts);
    let runner = |id| [fault(id, VoiceFault::Runner)];
    let path = Cow::Borrowed("m.gguf");
    let clip = Cow::Borrowed("c.wav");
    let started = VoiceAsk::Start {
        id: 1,
        setup: setup(true),
    };
    assert_eq!(rig.ask(started), runner(1));
    assert_eq!(rig.ask(VoiceAsk::Devices { id: 2 }), runner(2));
    assert_eq!(rig.ask(VoiceAsk::Facts { id: 3, path }), runner(3));
    assert_eq!(
        rig.ask(VoiceAsk::Check {
            id: 4,
            model: model(),
            clip
        }),
        runner(4)
    );
    assert_eq!(
        rig.ask(start(5)),
        [stage(5, Stage::Open)],
        "listening needs no engine"
    );
}

#[cfg(not(feature = "engine"))]
#[test]
fn a_build_without_the_engine_ships_no_engine() {
    let (tx, _rx) = mpsc::channel();
    let mut job = Job::new(Parts::live(), tx);
    let mut notes = Vec::new();
    let started = VoiceAsk::Start {
        id: 1,
        setup: setup(true),
    };
    for ask in [started, VoiceAsk::Devices { id: 2 }] {
        job.take(Event::Ask(ask), &mut notes).unwrap();
    }
    assert_eq!(
        notes,
        [fault(1, VoiceFault::Runner), fault(2, VoiceFault::Runner)]
    );
}
