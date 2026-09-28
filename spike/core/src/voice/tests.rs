//! Voice tests: the Mic button's cycle and what the caption bar shows.

use super::*;
use crate::voicecfg::Engine;

fn voice() -> VoiceConfig {
    crate::config::load().expect("spike.toml loads").voice
}

fn text(t: &str) -> Event {
    Event::Text {
        text: t.into(),
        engine: Engine::Local,
        ms: 1,
        audio_ms: 1,
    }
}

#[test]
fn the_mic_button_starts_then_stops_then_waits() {
    let mut b = MicButton::default();
    assert_eq!(b.click("ar"), Some(Cmd::Record { lang: "ar".into() }));
    assert_eq!(b.click("ar"), Some(Cmd::Stop));
    assert_eq!(
        b.click("ar"),
        None,
        "no click while the words are on their way"
    );
    b.on_event(&text("x"), &voice());
    assert_eq!(b.click("en"), Some(Cmd::Record { lang: "en".into() }));
}

#[test]
fn each_event_has_its_caption() {
    let (v, mut b) = (voice(), MicButton::default());
    let shown = |s: Step| s.caption.map(|c| (c.text, c.hide_ms));
    let listening = b.on_event(&Event::Listening, &v);
    assert!(
        listening.caption.as_ref().is_some_and(|c| c.rec),
        "the red dot"
    );
    assert_eq!(shown(listening), Some((v.caption.listening.clone(), None)));
    let busy = b.on_event(&Event::Transcribing { audio_ms: 9 }, &v);
    assert!(busy.caption.as_ref().is_some_and(|c| !c.rec));
    assert_eq!(shown(busy), Some((v.caption.transcribing.clone(), None)));
    assert_eq!(
        b.click("en"),
        None,
        "a recording that stopped by itself is busy too"
    );
    let err = b.on_event(
        &Event::Error {
            error: "no mic".into(),
        },
        &v,
    );
    assert_eq!(shown(err), Some(("no mic".into(), Some(v.caption.hide_ms))));
    assert!(
        b.on_event(&Event::Ready { load_ms: 5 }, &v)
            .caption
            .is_none()
    );
}

#[test]
fn the_words_are_shown_and_typed_with_the_separator() {
    let (v, mut b) = (voice(), MicButton::default());
    let s = b.on_event(&text("bonjour"), &v);
    assert_eq!(s.typed, Some(format!("bonjour{}", v.type_suffix)));
    assert_eq!(s.caption.map(|c| c.text), Some("bonjour".into()));
    assert!(
        b.on_event(&text(""), &v).typed.is_none(),
        "nothing heard, nothing typed"
    );
}

#[test]
fn the_button_follows_the_worker_and_notes_do_not_reset_it() {
    let (v, mut b) = (voice(), MicButton::default());
    b.click("en");
    let note = Event::Note {
        note: "clip not kept".into(),
    };
    assert!(b.on_event(&note, &v).caption.is_some(), "a note is shown");
    assert_eq!(
        b.click("en"),
        Some(Cmd::Stop),
        "a note leaves the recording going"
    );
    let mut late = MicButton::default();
    late.on_event(&Event::Listening, &v);
    assert_eq!(
        late.click("en"),
        Some(Cmd::Stop),
        "the worker says it records, so a click stops"
    );
}

#[test]
fn an_error_frees_the_button_and_a_note_keeps_it_busy() {
    let (v, mut b) = (voice(), MicButton::default());
    b.click("en");
    b.click("en");
    let note = Event::Note { note: "n".into() };
    b.on_event(&note, &v);
    assert_eq!(b.click("en"), None, "still waiting for the words");
    let error = Event::Error { error: "e".into() };
    b.on_event(&error, &v);
    assert_eq!(b.click("en"), Some(Cmd::Record { lang: "en".into() }));
}

#[test]
fn nothing_heard_says_so_and_types_nothing() {
    let (v, mut b) = (voice(), MicButton::default());
    let s = b.on_event(&text(" "), &v);
    assert_eq!(
        s.caption.map(|c| c.text),
        Some(v.caption.nothing_heard.clone())
    );
    assert!(s.typed.is_none());
}

#[test]
fn debug_never_shows_the_words() {
    let (v, mut b) = (voice(), MicButton::default());
    let shown = format!("{:?}", b.on_event(&text("secret words"), &v));
    assert!(!shown.contains("secret"), "{shown}");
}

#[test]
fn a_missing_worker_is_shown_and_the_button_stays_usable() {
    let mut cfg = crate::config::load().expect("spike.toml loads");
    cfg.voice.worker = "kx-no-such-worker.exe".into();
    let mut s = Session::start(&cfg, &[], |_| {});
    for _ in 0..2 {
        let c = s.click().caption;
        assert!(c.is_some_and(|c| c.text.contains("kx-no-such-worker.exe")));
    }
}

#[test]
fn the_mic_records_from_its_click_until_the_words_are_on_their_way() {
    let (v, mut b) = (voice(), MicButton::default());
    assert!(!b.recording());
    b.click("en");
    assert!(b.recording(), "the key turns red at once");
    b.on_event(&Event::Transcribing { audio_ms: 9 }, &v);
    assert!(!b.recording());
    b.on_event(&Event::Listening, &v);
    assert!(b.recording(), "the worker says it records");
    let error = Event::Error {
        error: "no mic".into(),
    };
    b.on_event(&error, &v);
    assert!(!b.recording());
}

#[test]
fn only_the_wait_for_the_words_is_busy() {
    let (v, mut b) = (voice(), MicButton::default());
    let busy = |s: Step| s.caption.is_some_and(|c| c.busy);
    assert!(!busy(b.on_event(&Event::Listening, &v)));
    assert!(busy(b.on_event(&Event::Transcribing { audio_ms: 9 }, &v)));
    assert!(!busy(b.on_event(&text("x"), &v)));
}
