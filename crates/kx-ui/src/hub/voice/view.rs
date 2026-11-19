//! The bar as QML reads it (design "The bar"): each state's words in the keyboard's language, its buttons in one order
//! in every language with the dimmed ones in place, a text with its own direction or dots, a note with its action, and
//! a service's card warning while the first dictation to it listens.
//! The gear shows in every state but `hidden`, before the last button.

use super::cloud::{self, CloudMark};
use super::panel::{self, Panel};
use super::parts::{Button, Hint, Lang, Tone};
use crate::I18n;
use crate::hub::wiped;
use crate::i18n::clean;
use crate::i18n::keys::vars::{MINUTES, SECONDS, SIZE};
use crate::i18n::keys::{
    FACT_WARN, VOICE_COPIED, VOICE_DOWNLOAD_SIZE, VOICE_GEAR, VOICE_LISTENING, VOICE_MASK,
    VOICE_NOTHING, VOICE_TIMER, VOICE_TITLE, VOICE_TRANSCRIBING, WINDOW_VOICE,
};
use kx_module_api::Zeroizing;
use kx_module_api::ports::{
    BarButton, BarHint, BarNote, BarPlace, BarText, Listening, LocalUse, VoiceBar, VoicePage,
    VoiceText,
};
use serde::Serialize;

/// The bar with its cloud mark, and the panel, as JSON.
#[derive(Serialize)]
pub(super) struct View {
    round: u32,
    place: BarPlace,
    sounds: bool,
    words: Words,
    bar: Bar,
    cloud: Option<CloudMark>,
    panel: Option<Panel>,
}

/// Qt's marks for the timer's minutes and seconds, which QML fills with `arg`.
const TIMER_MARKS: [&str; 2] = ["%1", "%2"];

/// The words outside the buttons: the window's and bar's names, the gear's, Copy's once it copied, and the timer
/// with `TIMER_MARKS` in its language's order.
#[derive(Serialize)]
struct Words {
    window: String,
    title: String,
    gear: String,
    copied: String,
    timer: String,
}

/// The bar's state, with its buttons in order.
#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
enum Bar {
    Hidden,
    #[serde(rename_all = "camelCase")]
    Listening {
        lang: Lang,
        word: String,
        elapsed_ms: u32,
        wave_ms: u16,
        hint: Option<String>,
        warn: Option<String>,
        buttons: Vec<Button>,
    },
    Transcribing {
        word: String,
        buttons: Vec<Button>,
    },
    Text {
        text: Shown,
        hint: Option<Hint>,
        buttons: Vec<Button>,
    },
    Note {
        hint: Hint,
        mic: bool,
        buttons: Vec<Button>,
    },
}

/// The text on the bar, wiped on drop, or the dots a password box's text shows as.
#[derive(Serialize)]
struct Shown {
    #[serde(serialize_with = "wiped::text")]
    text: Zeroizing<String>,
    rtl: bool,
    arabic: bool,
    masked: bool,
}

/// `page` with every text in `t`; `read` is Compare's set text in the language in front, if it has one.
pub(super) fn of(page: &VoicePage, t: &I18n, read: Option<&str>) -> View {
    View {
        round: page.round,
        place: page.place,
        sounds: page.sounds,
        words: Words {
            window: t.text(WINDOW_VOICE),
            title: t.text(VOICE_TITLE),
            gear: t.text(VOICE_GEAR),
            copied: t.text(VOICE_COPIED),
            timer: timer(t),
        },
        bar: bar(&page.bar, t),
        cloud: page.cloud.as_ref().map(|s| cloud::cloud_mark(t, s)),
        panel: page.panel.as_ref().map(|p| panel::of(p, t, read)),
    }
}

/// The timer in `t`'s words, its minutes and seconds left as `TIMER_MARKS`.
fn timer(t: &I18n) -> String {
    let [minutes, seconds] = TIMER_MARKS.map(str::to_owned);
    t.voice_with(VOICE_TIMER, &[(MINUTES, minutes), (SECONDS, seconds)])
}

/// The bar in `t`'s words.
fn bar(bar: &VoiceBar, t: &I18n) -> Bar {
    let cancel = || Button::bar(t, BarButton::Cancel);
    // The strongest button last, drawn dark (owner, 2026-11-14).
    let stop = || Button::bar(t, BarButton::Stop).tone(Tone::Primary);
    match bar {
        VoiceBar::Hidden => Bar::Hidden,
        VoiceBar::Listening(l) => listening(l, t, vec![cancel(), stop()]),
        VoiceBar::Transcribing => Bar::Transcribing {
            word: t.text(VOICE_TRANSCRIBING),
            buttons: vec![cancel(), stop().on(false)],
        },
        VoiceBar::Text(text) => text_bar(text, t),
        VoiceBar::Note(note) => note_bar(note, t),
    }
}

/// The Listening state, with the hint while it hears nothing yet and the service's card warning on its first dictation.
fn listening(l: &Listening, t: &I18n, buttons: Vec<Button>) -> Bar {
    Bar::Listening {
        lang: Lang::of(t, &l.lang),
        word: t.text(VOICE_LISTENING),
        elapsed_ms: l.elapsed_ms,
        wave_ms: l.wave_ms,
        hint: l.nothing.then(|| t.text(VOICE_NOTHING)),
        warn: l.warn.as_ref().map(|w| t.voice_fact(FACT_WARN, w).text),
        buttons,
    }
}

/// The Text state: Copy only for a shown text, Undo insert dimmed once it cannot act, then Close.
fn text_bar(text: &BarText, t: &I18n) -> Bar {
    let (shown, copy) = match &text.text {
        VoiceText::Shown { text, rtl, arabic } => {
            let shown = Shown {
                text: Zeroizing::new(clean(text.expose())),
                rtl: *rtl,
                arabic: *arabic,
                masked: false,
            };
            (shown, Some(Button::bar(t, BarButton::Copy)))
        }
        VoiceText::Masked => {
            let dots = Shown {
                text: Zeroizing::new(t.text(VOICE_MASK)),
                rtl: false,
                arabic: false,
                masked: true,
            };
            (dots, None)
        }
    };
    let undo = Button::bar(t, BarButton::Undo).on(text.undo);
    let buttons = copy
        .into_iter()
        .chain([undo, Button::bar(t, BarButton::Close)]);
    Bar::Text {
        text: shown,
        hint: text.hint.as_ref().map(|h| Hint::of(t, h)),
        buttons: buttons.collect(),
    }
}

/// The Note state: the action it names, a cloud note's local model, then Close.
fn note_bar(note: &BarNote, t: &I18n) -> Bar {
    let act = note.act.map(|b| action(b, &note.hint, t));
    let local = match note.hint {
        BarHint::Cloud { local, .. } => use_local(t, local),
        _ => Vec::new(),
    };
    let close = Button::bar(t, BarButton::Close);
    let buttons = act.into_iter().chain(local).chain([close]);
    Bar::Note {
        hint: Hint::of(t, &note.hint),
        mic: note.mic,
        buttons: buttons.collect(),
    }
}

/// A note's action `b`; Download names the size of the model it fetches.
fn action(b: BarButton, hint: &BarHint, t: &I18n) -> Button {
    let button = Button::bar(t, b).tone(Tone::Primary);
    match (b, hint) {
        (BarButton::Download, BarHint::NoModel { bytes, .. }) => sized(t, button, *bytes),
        _ => button,
    }
}

/// "Use the local model", dimmed unless one is ready, then Download when none is (edge 20).
fn use_local(t: &I18n, local: LocalUse) -> Vec<Button> {
    let fetch = match local {
        LocalUse::Download { bytes } => Some(sized(t, Button::bar(t, BarButton::Download), bytes)),
        LocalUse::Ready | LocalUse::Dimmed => None,
    };
    [Some(cloud::use_local(t, local)), fetch]
        .into_iter()
        .flatten()
        .collect()
}

/// Download `button` naming the size of the `bytes` it fetches.
fn sized(t: &I18n, button: Button, bytes: u64) -> Button {
    let size = (SIZE, t.voice_size(bytes));
    button.labelled(t.voice_with(VOICE_DOWNLOAD_SIZE, &[size]))
}
