//! Voice typing over the whole app: the module watches the session while it runs and Quit lets it go, the Dictation
//! key opens a dictation whose text the keyboard types and the voice shelf keeps, Paste at once pastes it through the
//! clipboard history and puts the clipboard back, and a hidden run still dictates but keeps nothing (ADR-0037, edge 16).

use crate::rig::{Rig, WAIT, hear};
use keyxtend::boot::Boot;
use keyxtend::hidden::Run;
use kx_mod_clipboard::ClipSettings;
use kx_mod_voice::models::table;
use kx_mod_voice::{INSERTS, VoiceSettings};
use kx_module_api::ModuleState::Active;
use kx_module_api::ports::{
    ClipPart, Drive, FaceAction, FieldKind, KeyId, LayoutChanged, LayoutId, MB, ShortcutId,
    SpeechDone, SpeechNote, SpeechStage, TextDone, TextPasted, VoiceLit,
};
use kx_module_api::{Event, Redacted, Zeroizing};
use kx_platform_fake::{ShelfCall, SpeechCall};
use std::path::Path;

/// The models folder the tests set.
const MODELS: &str = r"D:\KeyXtend\models";
/// The Dictation key.
const MIC: &str = "mic";
/// The language the tests dictate in, the rig's first layout.
const ENGLISH: &str = "en";
/// The app the hidden run was started from.
const APP: &str = "Claude";
/// What the test says.
const SAID: &str = "hello";
/// What the clipboard held before.
const KEPT: &str = "kept";

/// A rig whose models folder holds English's default model, on drive D:, with focus in a text box.
fn rig() -> Rig {
    let rig = Rig::new();
    rig.files.set_drives(vec![Drive {
        root: Redacted::new(r"D:\".into()),
        name: Redacted::new("D:".into()),
        free: 900_000 * MB,
    }]);
    let row = table().unwrap().default_for(ENGLISH).unwrap();
    let file = Path::new(MODELS).join(&row.file);
    rig.files.add_file(&file, row.bytes, row.print);
    rig.field.focus(FieldKind::Text);
    rig
}

/// Clicks the Dictation key on the keyboard.
fn tap_mic(boot: &Boot) {
    let tap = FaceAction::Tap {
        id: KeyId::new(MIC),
        back: false,
    };
    let _ = boot.hub.act(tap);
}

/// Starts `boot` and points voice typing at the models folder, with `insert` as its way in.
fn start(boot: &mut Boot, insert: &str) {
    boot.start().unwrap();
    for (key, value) in [("models_dir", MODELS), ("insert", insert)] {
        let value = toml::Value::String(value.into());
        boot.kernel
            .set_setting(kx_mod_voice::ID, key, value)
            .unwrap();
    }
}

/// Dictates `SAID` in English on a started `boot`; it gives the first `E` heard once the text is out.
fn speak<E: Event + Clone>(rig: &Rig, boot: &mut Boot) -> E {
    let bus = boot.kernel.bus();
    let ((_lit, lit), (_done, done)) = (hear::<VoiceLit>(&bus), hear::<E>(&bus));
    let english = LayoutChanged {
        id: LayoutId::new(1),
        iso: ENGLISH.into(),
        name: "English".into(),
        rtl: false,
    };
    bus.publish(&english).unwrap();
    rig.speech
        .on(SpeechCall::Open, vec![SpeechNote::Stage(SpeechStage::Open)]);
    tap_mic(boot);
    while lit.recv_timeout(WAIT).unwrap() != VoiceLit::Rec {}
    assert!(rig.speech.tell(SpeechNote::Heard));
    rig.clock()
        .advance(VoiceSettings::from_defaults().unwrap().debounce());
    tap_mic(boot);
    assert!(
        rig.speech.wait(WAIT, |runs| runs[0].stops == 1),
        "the key stops it"
    );
    let text = Redacted::new(Zeroizing::new(SAID.to_owned()));
    assert!(rig.speech.tell(SpeechNote::Piece { index: 0, text }));
    assert!(rig.speech.tell(SpeechNote::Done(SpeechDone {
        pieces: 1,
        sound_ms: 2_000,
        wait_ms: 300,
        load_ms: 0,
        memory_mb: 600,
        card: false,
    })));
    done.recv_timeout(WAIT).unwrap()
}

/// Starts `boot` and dictates `SAID` as `speak` does, typed; it gives the keyboard's answer.
fn dictate(rig: &Rig, boot: &mut Boot) -> TextDone {
    start(boot, INSERTS[0].0);
    speak(rig, boot)
}

#[test]
fn the_voice_module_watches_the_session_while_it_runs_and_quit_lets_it_go() {
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    assert_eq!(boot.kernel.state(kx_mod_voice::ID), Some(Active));
    assert!(
        rig.session.watching(),
        "a lock or a sleep stops a dictation"
    );
    boot.kernel.stop_all();
    assert!(!rig.session.watching());
}

#[test]
fn the_dictation_key_opens_a_dictation_whose_text_the_keyboard_types_and_the_shelf_keeps() {
    let rig = rig();
    let mut boot = rig.load();
    let saves = rig.voice.calls(ShelfCall::Save);
    let typed = dictate(&rig, &mut boot);
    assert!(typed.ok && typed.count > 0, "{typed:?}");
    assert!(!rig.injector.strokes().is_empty(), "the keyboard typed it");
    boot.kernel.stop_all();
    assert!(rig.voice.calls(ShelfCall::Save) > saves, "History keeps it");
}

#[test]
fn a_hidden_run_still_dictates_but_keeps_nothing() {
    let rig = rig();
    let run = Run::hidden(APP.into());
    let mut boot = rig.load_as(&run);
    let saves = rig.voice.calls(ShelfCall::Save);
    let typed = dictate(&rig, &mut boot);
    assert!(typed.ok && typed.count > 0, "{typed:?}");
    boot.kernel.stop_all();
    assert_eq!(rig.voice.calls(ShelfCall::Save), saves, "ADR-0037");
}

#[test]
fn paste_at_once_pastes_the_text_through_the_clipboard_history_then_puts_the_clipboard_back() {
    let rig = rig();
    rig.shortcuts.set_known(vec![crate::clip::paste()]);
    rig.clip.set_text(Some(KEPT));
    let mut boot = rig.load();
    start(&mut boot, INSERTS[1].0);
    let pasted: TextPasted = speak(&rig, &mut boot);
    assert!(pasted.ok, "{pasted:?}");
    let sent: Vec<ShortcutId> = rig.shortcuts.sent().into_iter().map(|s| s.0).collect();
    assert_eq!(sent, [crate::clip::paste().id], "one paste");
    assert!(rig.injector.strokes().is_empty(), "nothing typed");
    rig.clock()
        .advance(ClipSettings::from_defaults().unwrap().paste_gap());
    assert!(rig.writer.wait_tries(2, WAIT), "the clipboard goes back");
    let writes = rig.writer.writes();
    let said =
        |parts: &[ClipPart]| matches!(parts, [ClipPart::Text(t)] if t.expose().starts_with(SAID));
    assert!(said(&writes[0]), "the text, as ours");
    assert_eq!(writes[1], [ClipPart::Text(Redacted::new(KEPT.to_owned()))]);
    boot.kernel.stop_all();
}
