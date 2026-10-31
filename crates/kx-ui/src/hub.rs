//! The hub the `ui` module, the app and the QML bridges share: settings, the keyboard's and the system's texts, the
//! face, the look and its fonts, notices, sounds, the pointer overlay, the scroll pad, the chip lane, the clipboard
//! panel, the tray icon, and the requests the app drains on its own thread. No lock is held while a face call, a port
//! call or a waker runs.

mod app;
mod assist;
mod bus;
mod clip;
mod face;
mod fonts;
mod look;
mod notices;
mod scroll;
mod sound;
mod spots;
mod texts;
mod timings;
mod tray;
mod unread;
mod words;

pub use app::Mode;
#[cfg(feature = "qt")]
pub(crate) use clip::PICTURES;
pub use clip::acts as clip_acts;
pub use face::{Redraw, Resize};
pub use fonts::FontBook;
pub use notices::Shown;
pub use words::asks;

use crate::settings::UiSettings;
use kx_module_api::ports::KeyboardFace;
use kx_module_api::{ModuleId, SettingsError, SettingsHandle};
use kx_platform::ui::{
    Backdrop, FocusToken, Memory, Reveal, Screens, Speaker, SysLook, SystemLook, Tray, TrayEvent,
    WindowGuard,
};
use kx_theme::assets::FontFiles;
use kx_theme::{ThemeError, Themes};
use notices::Notices;
use serde::Serialize;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use texts::Texts;
use thiserror::Error;

/// The UI ports the app hands over; they run on the UI thread unless their docs say otherwise.
pub struct UiPorts {
    /// Keeps our windows from taking the focus and shapes where they take clicks.
    pub guard: Arc<dyn WindowGuard>,
    /// The screens and the pointer.
    pub screens: Arc<dyn Screens>,
    /// The system look and the system's display language.
    pub look: Arc<dyn SystemLook>,
    /// The material behind a window.
    pub backdrop: Arc<dyn Backdrop>,
    /// The tray icon, whose events come on the adapter's thread.
    pub tray: Arc<dyn Tray>,
    /// Gives the heaps' free pages back after a look change (ADR-0023).
    pub memory: Arc<dyn Memory>,
    /// Plays the sounds; any thread may call it, since it never blocks.
    pub speaker: Arc<dyn Speaker>,
    /// Shows the clipboard history's folder in the file manager; it may wait, so the hub calls it off the UI thread.
    pub reveal: Arc<dyn Reveal>,
}

/// What the UI asks of the app, which owns the kernel and drains these on its thread until the queue ends.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// Try again on a failed module.
    Retry(ModuleId),
    /// The user or a second copy chose Quit: stop the modules, then call `Hub::end_ui`; the queue ends after it.
    Quit,
}

/// What changed, so the bridges redraw on the Qt thread.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Wake {
    /// The `[ui]` settings or the face, and with them the texts: read everything again.
    Settings,
    /// The keyboard's language changed: read only its texts again.
    Texts,
    /// The look to draw.
    Look,
    /// Notices wait for the banner.
    Notices,
    /// A second copy asks to show the keyboard.
    Show,
    /// The app stopped the modules: the UI ends.
    End,
    /// The tray icon heard a click, with where its menu opens, or a click elsewhere.
    Tray(TrayEvent),
    /// The pointer overlay's state changed; no other comes until QML reads it.
    Assist,
    /// A mouse mode, a used latch, the scroll pad or a page turn changed the keyboard: it reads its state again.
    Keys,
    /// The scroll state or its outline changed; no other comes until QML reads it.
    Scroll,
    /// A new chip lane: the top bar reads `words_json` again.
    Words,
    /// The pad turns our open list a page, back or on: call `turn_page` on the Qt thread.
    Page {
        /// Turn back a page.
        back: bool,
    },
    /// The clipboard panel shows a new view or closes; no other comes until QML reads `clip_json`.
    Clip,
}

/// Hands a `Wake` to the Qt thread; it must not block.
pub type Waker = Box<dyn Fn(Wake) + Send + Sync>;

/// Why the hub could not start.
#[derive(Debug, Error)]
pub enum HubError {
    /// The embedded `[ui]` defaults are broken.
    #[error(transparent)]
    Settings(#[from] SettingsError),
    /// The embedded theme tokens are broken.
    #[error(transparent)]
    Theme(#[from] ThemeError),
}

/// What changes while the app runs.
struct Live {
    settings: UiSettings,
    texts: Texts,
    face: Option<Arc<dyn KeyboardFace>>,
    store: Option<SettingsHandle>,
    sys: SysLook,
    notices: Notices,
    /// `Show` and `End` wakes that came before the waker.
    pending: Vec<Wake>,
    waker: Option<Arc<Waker>>,
    /// The app ended the UI.
    ended: bool,
    /// How the user left the keyboard.
    mode: Mode,
    /// The app's queue, until Quit ends it.
    requests: Option<Sender<Request>>,
}

/// The shared UI state; the app makes one and hands it to the `ui` module and the bridges.
pub struct Hub {
    ports: UiPorts,
    themes: Themes,
    /// Each bundled font family's files.
    font_files: FontFiles,
    /// The window that had the focus before ours, so the guard can give it back.
    focus: FocusToken,
    live: Mutex<Live>,
    /// The tray icon while it shows; only the UI thread sets it.
    tray: Mutex<tray::TraySlot>,
    /// The font book and the files it holds; only the UI thread uses it.
    fonts: Mutex<fonts::Fonts>,
    /// True once the memory port refused a give-back, which warns only the first time.
    refused: AtomicBool,
    /// Each sound's cue and turn.
    sounds: sound::Sounds,
    /// The pointer overlay's state.
    assist: assist::Assist,
    /// The scroll state and the box to outline.
    scroll: scroll::Scroll,
    /// The kernel's bus and the keyboard's wake handlers.
    heard: Mutex<bus::Heard>,
    /// The chip lane and the handle that sends its taps.
    words: Mutex<words::Words>,
    /// The clipboard panel and the history's folder.
    clip: clip::Clip,
}

impl Hub {
    /// A hub on the `[ui]` defaults until the `ui` module starts, and the requests the app drains.
    /// Call it before any window of ours exists, so the focus it notes is the user's app.
    ///
    /// # Errors
    /// `HubError` when the embedded defaults or theme tokens are broken.
    pub fn new(ports: UiPorts) -> Result<(Arc<Self>, Receiver<Request>), HubError> {
        let settings = UiSettings::from_defaults()?;
        let themes = Themes::shipped()?;
        let font_files = FontFiles::shipped()?;
        let sounds = sound::Sounds::new(&*ports.speaker, settings.sound_idle_ms);
        let (requests, drain) = mpsc::channel();
        let live = Live {
            texts: Texts::new(&settings.language, "", &ports.look.ui_language()),
            settings,
            face: None,
            store: None,
            sys: ports.look.read(),
            notices: Notices::default(),
            pending: Vec::new(),
            waker: None,
            ended: false,
            mode: Mode::Shown,
            requests: Some(requests),
        };
        let hub = Self {
            focus: ports.guard.remember(),
            ports,
            themes,
            font_files,
            live: Mutex::new(live),
            tray: Mutex::default(),
            fonts: Mutex::default(),
            refused: AtomicBool::default(),
            sounds,
            assist: assist::Assist::default(),
            scroll: scroll::Scroll::default(),
            heard: Mutex::default(),
            words: Mutex::default(),
            clip: clip::Clip::default(),
        };
        Ok((Arc::new(hub), drain))
    }

    fn lock(&self) -> MutexGuard<'_, Live> {
        self.live.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Takes the `ui` module's settings, the keyboard face and the module's own settings handle; the keyboard's texts
    /// follow the language of the layout the face shows, a new `sound_idle_ms` prepares the speaker again, and an open
    /// clipboard panel hears the rows. The keyboard's wake is heard from now on, after the face's own handlers.
    pub fn attach(
        self: &Arc<Self>,
        settings: UiSettings,
        face: Arc<dyn KeyboardFace>,
        store: SettingsHandle,
    ) {
        self.hear_keys();
        let texts = Texts::new(
            &settings.language,
            &face.lang(),
            &self.ports.look.ui_language(),
        );
        let idle = settings.sound_idle_ms;
        let before = {
            let mut live = self.lock();
            live.texts = texts;
            live.face = Some(face);
            live.store = Some(store);
            std::mem::replace(&mut live.settings, settings).sound_idle_ms
        };
        self.idle_changed(before, idle);
        self.clip_rows();
        self.wake(Wake::Settings);
    }

    /// Lets the face, the handle and the keyboard's wake go when the `ui` module stops; the last settings stay.
    pub fn detach(&self) {
        self.unhear_keys();
        {
            let mut live = self.lock();
            live.face = None;
            live.store = None;
        }
        self.wake(Wake::Settings);
    }

    /// The `[ui]` settings now.
    #[must_use]
    pub fn settings(&self) -> UiSettings {
        self.lock().settings.clone()
    }

    /// Calls the waker, if any, outside the lock.
    fn wake(&self, what: Wake) {
        let waker = self.lock().waker.clone();
        if let Some(waker) = waker {
            waker(what);
        }
    }
}

/// `value` as JSON; a value that cannot be written gives `null`.
pub(crate) fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| NULL.to_owned())
}

/// JSON for nothing.
pub(crate) const NULL: &str = "null";
