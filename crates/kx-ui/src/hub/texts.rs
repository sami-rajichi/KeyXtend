//! The hub's two text sets (ADR-0020): the keyboard's, which follow the language of the layout in front unless
//! `[ui] language` pins one, and the system's, which the tray icon and its menu speak.

use super::{Hub, Live, Shown, Wake, json};
use crate::i18n::keys::{SIZE, SIZE_NAME, SIZES};
use crate::{I18n, Lang, os_lang, pick};
use kx_module_api::ports::KeyboardFace;
use serde::Serialize;
use std::sync::Arc;

/// The keyboard's texts and the system's.
pub(super) struct Texts {
    /// The keyboard's texts.
    keyboard: Arc<I18n>,
    /// The system's texts.
    tray: Arc<I18n>,
}

impl Texts {
    /// The texts for `[ui] language` value `setting`, a layout in language `layout` and a system in `os`.
    pub(super) fn new(setting: &str, layout: &str, os: &str) -> Self {
        Self {
            keyboard: Arc::new(I18n::new(pick(setting, layout))),
            tray: Arc::new(I18n::new(os_lang(os))),
        }
    }

    /// The keyboard's texts.
    pub(super) fn keyboard(&self) -> &I18n {
        &self.keyboard
    }

    /// Follows `setting` and a layout in language `layout`; true when the keyboard's language changed.
    fn follow(&mut self, setting: &str, layout: &str) -> bool {
        let lang = pick(setting, layout);
        if lang.tag == self.keyboard.lang().tag {
            return false;
        }
        self.keyboard = Arc::new(I18n::new(lang));
        true
    }
}

/// A language's tag and direction, as QML reads them.
#[derive(Serialize)]
struct Tag {
    tag: &'static str,
    rtl: bool,
}

impl From<&Lang> for Tag {
    fn from(l: &Lang) -> Self {
        Self {
            tag: l.tag,
            rtl: l.rtl,
        }
    }
}

/// The keyboard's language with its size buttons' texts, and the tray's language.
#[derive(Serialize)]
struct LangView {
    #[serde(flatten)]
    keyboard: Tag,
    sizes: Vec<SizeText>,
    tray: Tag,
}

/// A size button's id in `SIZES`, its letter and its name.
#[derive(Serialize)]
struct SizeText {
    id: &'static str,
    label: String,
    name: String,
}

impl Hub {
    /// The keyboard's texts now.
    #[must_use]
    pub fn i18n(&self) -> Arc<I18n> {
        Arc::clone(&self.lock().texts.keyboard)
    }

    /// The system's texts, which the tray icon and its menu speak.
    #[must_use]
    pub fn tray_i18n(&self) -> Arc<I18n> {
        Arc::clone(&self.lock().texts.tray)
    }

    /// The keyboard's language, its size buttons' letters and names, and the tray's language, as JSON for QML.
    #[must_use]
    pub fn lang_json(&self) -> String {
        let (t, tray) = (self.i18n(), self.tray_i18n());
        let size = |id: &'static str| SizeText {
            id,
            label: t.name_of(SIZE, id),
            name: t.name_of(SIZE_NAME, id),
        };
        json(&LangView {
            keyboard: t.lang().into(),
            sizes: SIZES.into_iter().map(size).collect(),
            tray: tray.lang().into(),
        })
    }

    /// The notices the banner shows, told again in the keyboard's language now.
    #[must_use]
    pub fn shown_notices(&self) -> Vec<Shown> {
        let live = self.lock();
        live.notices.shown(&live.texts.keyboard)
    }

    /// Follows the language of the layout `face` shows; a new one wakes the UI to read the keyboard's texts again.
    pub(super) fn follow(&self, face: &dyn KeyboardFace) {
        let layout = face.lang();
        let changed = {
            let mut live = self.lock();
            let Live {
                settings, texts, ..
            } = &mut *live;
            texts.follow(&settings.language, &layout)
        };
        if changed {
            self.wake(Wake::Texts);
        }
    }
}
