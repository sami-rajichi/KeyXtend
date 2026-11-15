//! The bar as QML reads it (design "The bar"): each state's words in the keyboard's language, its buttons in one order
//! in every language with the dimmed ones in place, a text with its own direction or dots, and a note with its action.
//! The gear shows in every state but `hidden`, before the last button.

use super::panel::{self, Panel};
use super::parts::{Button, Hint, Lang, Tone};
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::SIZE;
use crate::i18n::keys::{
    VOICE_COPIED, VOICE_DOWNLOAD_SIZE, VOICE_GEAR, VOICE_LISTENING, VOICE_MASK, VOICE_NOTHING,
    VOICE_TITLE, VOICE_TRANSCRIBING, WINDOW_VOICE,
};
use kx_module_api::ports::{
    BarButton, BarHint, BarNote, BarPlace, BarText, Listening, VoiceBar, VoicePage, VoiceText,
};
use serde::Serialize;

/// The bar and the panel, as JSON.
#[derive(Serialize)]
pub(super) struct View {
    round: u32,
    place: BarPlace,
    sounds: bool,
    words: Words,
    bar: Bar,
    panel: Option<Panel>,
}

/// The words outside the buttons: the window's and bar's names, the gear's, and Copy's once it copied.
#[derive(Serialize)]
struct Words {
    window: String,
    title: String,
    gear: String,
    copied: String,
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

/// The text on the bar, or the dots a password box's text shows as.
#[derive(Serialize)]
struct Shown {
    text: String,
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
        },
        bar: bar(&page.bar, t),
        panel: page.panel.as_ref().map(|p| panel::of(p, t, read)),
    }
}

/// The bar in `t`'s words.
fn bar(bar: &VoiceBar, t: &I18n) -> Bar {
    let cancel = || Button::bar(t, BarButton::Cancel);
    let stop = || Button::bar(t, BarButton::Stop);
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

/// The Listening state, with the hint while it hears nothing yet.
fn listening(l: &Listening, t: &I18n, buttons: Vec<Button>) -> Bar {
    Bar::Listening {
        lang: (&l.lang).into(),
        word: t.text(VOICE_LISTENING),
        elapsed_ms: l.elapsed_ms,
        wave_ms: l.wave_ms,
        hint: l.nothing.then(|| t.text(VOICE_NOTHING)),
        buttons,
    }
}

/// The Text state: Copy only for a shown text, Undo insert dimmed once it cannot act, then Close.
fn text_bar(text: &BarText, t: &I18n) -> Bar {
    let (shown, copy) = match &text.text {
        VoiceText::Shown { text, rtl, arabic } => {
            let shown = Shown {
                text: clean(text.expose()),
                rtl: *rtl,
                arabic: *arabic,
                masked: false,
            };
            (shown, Some(Button::bar(t, BarButton::Copy)))
        }
        VoiceText::Masked => {
            let dots = Shown {
                text: t.text(VOICE_MASK),
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

/// The Note state: the action it names, then Close.
fn note_bar(note: &BarNote, t: &I18n) -> Bar {
    let act = note.act.map(|b| action(b, &note.hint, t));
    let buttons = act.into_iter().chain([Button::bar(t, BarButton::Close)]);
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
        (BarButton::Download, BarHint::NoModel { bytes, .. }) => {
            let size = (SIZE, t.voice_size(*bytes));
            button.labelled(t.voice_with(VOICE_DOWNLOAD_SIZE, &[size]))
        }
        _ => button,
    }
}
