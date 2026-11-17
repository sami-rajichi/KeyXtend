//! The engine over a fake library: pieces in order in the model's own language, the threads rule (edge case 30), one
//! model per job, refusals and a dropped writer. Notes are awaited on their channel; nothing sleeps.

mod asks;
mod card;

use super::*;
use crate::voice::Event;
use crate::voice::fake::lib::{FakeLib, device};
use crate::voice::library::{Failed, Place};
use kx_ipc::{DeviceKind, Stage};
use std::time::Duration;

/// The longest the tests wait for one note, far past what the fake engine needs.
const WAIT: Duration = Duration::from_secs(30);

/// An existing file to name as a model: the test program itself.
fn file() -> String {
    std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

/// A model on `target` in `lang` with `threads`.
fn model(target: Target<'static>, lang: &'static str, threads: u8) -> Model<'static> {
    Model {
        path: Cow::Owned(file()),
        lang: Cow::Borrowed(lang),
        runner: Cow::Borrowed(FakeLib::RUNNER),
        target,
        threads,
    }
}

fn card(need_mb: u32) -> Target<'static> {
    Target::Card {
        id: Cow::Borrowed("Vulkan0"),
        need_mb,
    }
}

/// An engine on 8 cores over a fake library with the processor and `Vulkan0`, `free_mb` free, knowing English and
/// French; the library, and where the notes come.
fn rig(free_mb: u64) -> (Local<FakeLib>, FakeLib, Tell, Receiver<Event>) {
    let lib = FakeLib::default();
    let mut s = lib.script();
    s.found = vec![
        device(DeviceKind::Cpu, "CPU", 8000),
        device(DeviceKind::Card, "Vulkan0", free_mb),
    ];
    s.langs = vec!["en".into(), "fr-FR".into()];
    drop(s);
    let (tx, rx) = mpsc::channel();
    (Local::new(lib.clone(), 8), lib, Tell::new(tx), rx)
}

/// The next note, waiting at most `WAIT`.
fn next(rx: &Receiver<Event>) -> VoiceNote {
    loop {
        if let Event::Note(note) = rx.recv_timeout(WAIT).expect("a note within the wait") {
            return note;
        }
    }
}

/// The notes up to and with run `id`'s end: its done note or a fault that ends it.
fn run_notes(rx: &Receiver<Event>, id: u32) -> Vec<VoiceNote> {
    let mut out = Vec::new();
    loop {
        let note = next(rx);
        let end = match &note {
            VoiceNote::Done { .. } => true,
            VoiceNote::Fault { fault, .. } => !matches!(
                fault,
                VoiceFault::CardRoom | VoiceFault::CardFailed | VoiceFault::NoCard
            ),
            _ => false,
        };
        assert_eq!(note.id(), id);
        out.push(note);
        if end {
            return out;
        }
    }
}

/// `n` samples of sound.
fn sound(n: usize) -> Buf<i16> {
    Buf::collect(std::iter::repeat_n(100_i16, n)).unwrap()
}

fn loading(id: u32) -> VoiceNote {
    VoiceNote::Stage {
        id,
        stage: Stage::Loading,
    }
}

fn fault(id: u32, fault: VoiceFault) -> VoiceNote {
    VoiceNote::Fault { id, fault }
}

fn piece(id: u32, index: u16, text: &str) -> VoiceNote {
    let text = zeroize::Zeroizing::new(text.to_owned());
    VoiceNote::Piece { id, index, text }
}

/// Writes pieces of `lens` samples for run `id` on `model`, then ends it; its notes.
fn write(
    engine: &mut Local<FakeLib>,
    tell: &Tell,
    rx: &Receiver<Event>,
    id: u32,
    model: &Model<'_>,
    lens: &[usize],
) -> Vec<VoiceNote> {
    let mut pen = engine.load(id, model, tell.clone()).unwrap();
    for &n in lens {
        pen.piece(sound(n));
    }
    pen.end(1234);
    run_notes(rx, id)
}

/// The done note's pieces, load time and card flag.
fn done(note: &VoiceNote) -> (u16, u32, u32, bool) {
    let VoiceNote::Done {
        pieces,
        sound_ms,
        load_ms,
        card,
        ..
    } = note
    else {
        panic!("not done: {note:?}");
    };
    (*pieces, *sound_ms, *load_ms, *card)
}

#[test]
fn pieces_are_told_in_order_in_the_models_own_language() {
    let (mut engine, lib, tell, rx) = rig(0);
    let notes = write(
        &mut engine,
        &tell,
        &rx,
        1,
        &model(Target::Cpu, "fr-FR", 0),
        &[100, 200, 300],
    );
    let want = [
        loading(1),
        piece(1, 0, "100 fr-FR"),
        piece(1, 1, "200 fr-FR"),
        piece(1, 2, "300 fr-FR"),
    ];
    assert_eq!(notes[..4], want);
    assert_eq!(done(&notes[4]).0, 3);
    assert_eq!(done(&notes[4]).1, 1234);
    assert!(
        lib.script()
            .runs
            .iter()
            .all(|r| r.2 == "fr-FR" && r.0 == Place::Cpu)
    );
}

#[test]
fn threads_zero_is_the_real_cores_and_the_hosts_count_is_taken() {
    for (asked, want) in [(0, 8), (4, 4), (14, 14), (64, 64)] {
        let (mut engine, lib, tell, rx) = rig(0);
        write(
            &mut engine,
            &tell,
            &rx,
            1,
            &model(Target::Cpu, "en", asked),
            &[10],
        );
        assert_eq!(lib.script().loads, [(Place::Cpu, want)], "{asked}");
    }
}

#[test]
fn one_job_serves_one_model_file_and_keeps_it_loaded() {
    let (mut engine, lib, tell, rx) = rig(0);
    let m = model(Target::Cpu, "en", 0);
    write(&mut engine, &tell, &rx, 1, &m, &[10]);
    let again = write(&mut engine, &tell, &rx, 2, &m, &[20]);
    assert_eq!(again[0], piece(2, 0, "20 en"), "no second load");
    assert_eq!(done(&again[1]).2, 0);
    assert_eq!(lib.script().loads.len(), 1);
    let mut other = m.clone();
    other.path = Cow::Owned(format!("{}.other", file()));
    std::fs::write(&*other.path, b"x").unwrap();
    let refused = engine.load(3, &other, tell.clone()).err();
    std::fs::remove_file(&*other.path).unwrap();
    assert_eq!(refused, Some(VoiceFault::OutOfStep));
}

#[test]
fn another_runners_model_or_a_missing_file_is_refused_at_once() {
    let (mut engine, _, tell, _) = rig(0);
    let mut m = model(Target::Cpu, "en", 0);
    m.runner = Cow::Borrowed("other");
    assert_eq!(
        engine.load(1, &m, tell.clone()).err(),
        Some(VoiceFault::Runner)
    );
    let mut m = model(Target::Cpu, "en", 0);
    m.path = Cow::Owned(format!("{}.none", file()));
    assert_eq!(
        engine.load(1, &m, tell.clone()).err(),
        Some(VoiceFault::NoFile)
    );
    assert_eq!(
        engine.facts(2, &m.path, tell.clone()),
        Err(VoiceFault::NoFile)
    );
    let cpu = model(Target::Cpu, "en", 0);
    assert_eq!(
        engine.check(3, &cpu, &file(), tell),
        Err(VoiceFault::OutOfStep)
    );
}

#[test]
fn a_model_that_does_not_know_the_language_ends_the_run() {
    let (mut engine, _, tell, rx) = rig(0);
    let notes = write(
        &mut engine,
        &tell,
        &rx,
        1,
        &model(Target::Cpu, "ar", 0),
        &[10],
    );
    assert_eq!(notes, [loading(1), fault(1, VoiceFault::Language)]);
}

#[test]
fn a_dropped_writer_tells_no_piece_and_no_end() {
    let (mut engine, lib, tell, rx) = rig(0);
    let held = lib.script();
    let mut pen = engine
        .load(1, &model(Target::Cpu, "en", 0), tell.clone())
        .unwrap();
    pen.piece(sound(10));
    pen.end(10);
    drop(pen);
    drop(held);
    engine.devices(2, tell).unwrap();
    let mut seen = vec![next(&rx)];
    if seen[0] == loading(1) {
        seen.push(next(&rx));
    }
    assert!(
        matches!(seen.last(), Some(VoiceNote::Devices { id: 2, .. })),
        "{seen:?}"
    );
    assert!(
        lib.script().runs.is_empty(),
        "a stopped run's piece is never written"
    );
}
