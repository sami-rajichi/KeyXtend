//! The pieces the bar and the panel share as QML reads them: a button with the click it sends, a note's line, a
//! language and a line above a list. Each user string loses its hidden marks first (ADR-0022).

use super::acts::{BAR, BUTTON};
use crate::I18n;
use crate::i18n::keys::{VOICE_JOIN, VOICE_WORD, bar_button_id, bar_hint_id, panel_button_id};
use kx_module_api::ports::{BarButton, BarHint, PanelButton, VoiceLang};
use serde::Serialize;

/// How a button stands out: plain, the main action, or one that deletes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Tone {
    Plain,
    Primary,
    Danger,
}

/// A button, with the `kind`, `id` and `arg` QML hands back to `voice_act`; a dimmed one stays in its place.
#[derive(Serialize)]
pub(super) struct Button {
    kind: &'static str,
    id: &'static str,
    arg: i32,
    label: String,
    /// What readers hear when the label alone cannot tell it from its neighbours.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    on: bool,
    tone: Tone,
    /// The bar icon it wears in place of its id's, by its name in the theme's `[voice]` icons.
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<&'static str>,
}

impl Button {
    /// A plain button that acts, sending `kind` with `id` and `arg`.
    pub(super) const fn new(kind: &'static str, id: &'static str, arg: i32, label: String) -> Self {
        Self {
            kind,
            id,
            arg,
            label,
            name: None,
            on: true,
            tone: Tone::Plain,
            icon: None,
        }
    }

    /// Bar button `b` with its word.
    pub(super) fn bar(t: &I18n, b: BarButton) -> Self {
        let id = bar_button_id(b);
        Self::new(BAR, id, 0, t.text(&word(id)))
    }

    /// Panel button `b` labelled `label`, acting while `acts` holds it.
    pub(super) fn panel_as(b: PanelButton, label: String, acts: &[PanelButton]) -> Self {
        Self::new(BUTTON, panel_button_id(b), 0, label).on(acts.contains(&b))
    }

    /// Panel button `b` with its own word, acting while `acts` holds it.
    pub(super) fn panel(t: &I18n, b: PanelButton, acts: &[PanelButton]) -> Self {
        Self::panel_as(b, t.text(&word(panel_button_id(b))), acts)
    }

    /// This button, acting or dimmed.
    pub(super) const fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// This button in `tone`.
    pub(super) const fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// This button heard by readers as `name`, its label still shown.
    pub(super) fn named(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// This button wearing bar icon `icon` in place of its id's.
    pub(super) const fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// This button labelled `label`.
    pub(super) fn labelled(mut self, label: String) -> Self {
        self.label = label;
        self
    }
}

/// A note's line, by its id.
#[derive(Serialize)]
pub(super) struct Hint {
    id: &'static str,
    text: String,
}

impl Hint {
    /// `hint`'s line.
    pub(super) fn of(t: &I18n, hint: &BarHint) -> Self {
        Self {
            id: bar_hint_id(hint),
            text: t.voice_hint(hint),
        }
    }

    /// The line `text`, by `id`.
    pub(super) const fn new(id: &'static str, text: String) -> Self {
        Self { id, text }
    }
}

/// A language as the bar's chip and the panel name it, as a title.
#[derive(Serialize)]
pub(super) struct Lang {
    name: String,
    rtl: bool,
    arabic: bool,
}

impl Lang {
    /// Language `l`, named in `t`'s words.
    pub(super) fn of(t: &I18n, l: &VoiceLang) -> Self {
        Self {
            name: t.voice_title(l),
            rtl: l.rtl,
            arabic: l.arabic,
        }
    }
}

/// The ids of the lines above a list, which pick their icons: the best model, a failed check, Compare's step and what
/// Compare keeps.
pub(super) const TOP_BEST: &str = "best";
pub(super) const TOP_FAILED: &str = "failed";
pub(super) const TOP_STEP: &str = "step";
pub(super) const TOP_KEPT: &str = "kept";

/// A line above a list, by its id, with a name drawn apart, such as the model writing.
#[derive(Serialize)]
pub(super) struct Top {
    id: &'static str,
    text: String,
    name: Option<String>,
}

impl Top {
    /// The line `text`, by `id`, with `name` if any.
    pub(super) const fn new(id: &'static str, text: String, name: Option<String>) -> Self {
        Self { id, text, name }
    }
}

/// The voice key of word `id`.
pub(super) fn word(id: &str) -> String {
    format!("{VOICE_WORD}{id}")
}

/// The line readers hear of `first` then `then`, joined in `t`'s language.
pub(super) fn heard(t: &I18n, first: &str, then: &str) -> String {
    [first, then].join(&t.text(VOICE_JOIN))
}
