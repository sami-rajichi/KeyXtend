//! The face's side of voice: the Mic button, what the caption bar shows, and one session per face.

use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

use crate::config::SpikeConfig;
use crate::hold::Pt;
use crate::selwatch::PillPx;
use crate::typer::Typer;
use crate::voicecfg::VoiceConfig;
pub use crate::voiceproto::{Cmd, Event};
use crate::voiceworker::{WORKER_GONE, Worker};
use crate::{inject, layout, screen};

/// Where the Mic button is in its cycle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Phase {
    /// Ready to record.
    #[default]
    Idle,
    /// Recording; the next click stops.
    Recording,
    /// Waiting for the words; clicks do nothing.
    Busy,
}

/// The Mic button's state; it follows the worker's events.
#[derive(Debug, Default)]
pub struct MicButton {
    phase: Phase,
}

/// What the caption bar shows. `Debug` shows only the length, since the text may be dictated words.
#[derive(Clone, PartialEq, Eq)]
pub struct Caption {
    /// The text.
    pub text: String,
    /// Hide it after this many ms; `None` keeps it up.
    pub hide_ms: Option<u64>,
}

impl Caption {
    fn stay(text: &str) -> Caption {
        Caption {
            text: text.into(),
            hide_ms: None,
        }
    }

    fn brief(text: &str, hide_ms: u64) -> Caption {
        Caption {
            text: text.into(),
            hide_ms: Some(hide_ms),
        }
    }
}

impl std::fmt::Debug for Caption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.text.chars().count();
        write!(f, "Caption {{ {n} chars, hide {:?} }}", self.hide_ms)
    }
}

/// What the face does after an event. `Debug` hides the words.
#[derive(Default)]
pub struct Step {
    /// New caption, if any.
    pub caption: Option<Caption>,
    /// Text to type into the app in front.
    pub typed: Option<String>,
}

impl std::fmt::Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let typed = self.typed.as_ref().map(|t| t.chars().count());
        write!(f, "Step {{ {:?}, typed chars {typed:?} }}", self.caption)
    }
}

impl MicButton {
    /// The command for a click, in language `lang`; `None` while the words are on their way.
    pub fn click(&mut self, lang: &str) -> Option<Cmd> {
        let (next, cmd) = match self.phase {
            Phase::Idle => (Phase::Recording, Cmd::Record { lang: lang.into() }),
            Phase::Recording => (Phase::Busy, Cmd::Stop),
            Phase::Busy => return None,
        };
        self.phase = next;
        Some(cmd)
    }

    /// The caption and typing for event `e`; a note leaves the cycle as it is, an error ends it.
    pub fn on_event(&mut self, e: &Event, v: &VoiceConfig) -> Step {
        let (c, hide) = (&v.caption, v.caption.hide_ms);
        let (phase, caption, typed) = match e {
            Event::Ready { .. } => return Step::default(),
            Event::Listening => (Some(Phase::Recording), Caption::stay(&c.listening), None),
            Event::Transcribing { .. } => (Some(Phase::Busy), Caption::stay(&c.transcribing), None),
            Event::Text { text, .. } if text.trim().is_empty() => (
                Some(Phase::Idle),
                Caption::brief(&c.nothing_heard, hide),
                None,
            ),
            Event::Text { text, .. } => {
                let typed = format!("{text}{}", v.type_suffix);
                (Some(Phase::Idle), Caption::brief(text, hide), Some(typed))
            }
            Event::Note { note } => (None, Caption::brief(note, hide), None),
            Event::Error { error } => (Some(Phase::Idle), Caption::brief(error, hide), None),
        };
        if let Some(p) = phase {
            self.phase = p;
        }
        Step {
            caption: Some(caption),
            typed,
        }
    }
}

/// What the face shows after a click or an event.
#[derive(Debug, Default)]
pub struct Update {
    /// New caption, if any.
    pub caption: Option<Caption>,
    /// A problem for the status line, such as typing that failed.
    pub note: Option<String>,
}

/// Voice for one face: the Mic button, the worker (or why it did not start) and the typing thread.
pub struct Session {
    cfg: VoiceConfig,
    button: MicButton,
    worker: Result<Worker, String>,
    typer: Typer,
}

impl Session {
    /// Starts the worker at launch, so the model is loaded before the first click; events go to `on_event`.
    /// `face_args` are the face's own switches, which may carry the keep folder.
    pub fn start(
        cfg: &SpikeConfig,
        face_args: &[String],
        on_event: impl FnMut(Event) + Send + 'static,
    ) -> Session {
        Session {
            cfg: cfg.voice.clone(),
            button: MicButton::default(),
            worker: Worker::start(cfg, face_args, on_event),
            typer: Typer::start(cfg.voice.type_gap_ms, Box::new(inject::text_paced)),
        }
    }

    /// A Mic click: starts or stops recording in the language of the app in front.
    pub fn click(&mut self) -> Update {
        let lang = front_lang(&self.cfg);
        let Some(cmd) = self.button.click(&lang) else {
            return Update::default();
        };
        let sent = match &mut self.worker {
            Ok(w) => w.send(&cmd),
            Err(why) => Err(why.clone()),
        };
        match sent {
            Ok(()) => Update::default(),
            Err(error) => self.on_event(&Event::Error { error }),
        }
    }

    /// Queues the words of `e` for typing into the app in front and says what the caption bar shows.
    /// A typing error shows with the next event.
    pub fn on_event(&mut self, e: &Event) -> Update {
        let step = self.button.on_event(e, &self.cfg);
        let queued = step.typed.and_then(|t| self.typer.send(t).err());
        Update {
            caption: step.caption,
            note: queued.or_else(|| self.typer.take_note()),
        }
    }

    /// Asks the worker to quit; later clicks say it stopped.
    pub fn stop(&mut self) {
        self.worker = Err(WORKER_GONE.to_string());
    }
}

/// The spoken language for the layout of the app in front.
pub fn front_lang(v: &VoiceConfig) -> String {
    let hkl = layout::foreground_layout().0 as isize;
    v.language(layout::lang_id(hkl)).to_string()
}

/// Where the caption bar goes and its size, in physical pixels, on the screen of the app in front.
/// `px` is the width, height and bottom gap in logical pixels, converted like the pill's.
pub fn bar_place(px: [f32; 3]) -> Result<(Pt, (i32, i32)), String> {
    let mut r = RECT::default();
    // SAFETY: plain query; on failure `r` stays empty and the nearest screen is used.
    let _ = unsafe { GetWindowRect(GetForegroundWindow(), &mut r) };
    let work = screen::work_area(&r)?;
    let size = PillPx(px).at(screen::dpi_of(&r));
    let at = screen::bottom_centre(&work, (size.w, size.h), size.gap);
    Ok((at, (size.w, size.h)))
}

#[cfg(test)]
mod tests {
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
        assert_eq!(shown(listening), Some((v.caption.listening.clone(), None)));
        let busy = b.on_event(&Event::Transcribing { audio_ms: 9 }, &v);
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
}
