//! The hub the `ui` module, the app and the QML bridges share: settings, the keyboard's and the system's texts, the
//! face, the look and its fonts, notices, sounds, the tray icon, and the requests the app drains on its own thread. No
//! lock is held while a face call, a port call or a waker runs.

mod app;
mod face;
mod fonts;
mod look;
mod notices;
mod sound;
mod spots;
mod texts;
mod tray;

pub use app::Mode;
pub use face::{Redraw, Resize};
pub use fonts::FontBook;
pub use notices::Shown;

use crate::settings::{PressFrame, UiSettings};
use kx_module_api::ports::{AssistNote, KeyboardFace};
use kx_module_api::{
    Bus, Event, EventError, ModuleId, Notice, SettingsError, SettingsHandle, Subscription,
};
use kx_platform::ui::{
    Backdrop, FocusToken, Memory, Screens, Speaker, SysLook, SystemLook, Tray, TrayEvent,
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

/// The timings, paging, press frame and frame log QML reads, as JSON.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QmlSettings {
    tip_ms: u64,
    poll_ms: u64,
    guard_delay_ms: u64,
    tray_grace_ms: u64,
    page_rows: usize,
    press_frame: PressFrame,
    press_frame_ms: u64,
    frame_log: bool,
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
        };
        Ok((Arc::new(hub), drain))
    }

    fn lock(&self) -> MutexGuard<'_, Live> {
        self.live.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Takes the `ui` module's settings, the keyboard face and the module's own settings handle; the keyboard's texts
    /// follow the language of the layout the face shows, and a new `sound_idle_ms` prepares the speaker again.
    pub fn attach(&self, settings: UiSettings, face: Arc<dyn KeyboardFace>, store: SettingsHandle) {
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
        self.wake(Wake::Settings);
    }

    /// Lets the face and the handle go when the `ui` module stops; the last settings stay.
    pub fn detach(&self) {
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

    /// The timings, paging, press frame and frame log QML reads, as JSON; the language comes apart, in `lang_json`.
    #[must_use]
    pub fn settings_json(&self) -> String {
        let s = self.settings();
        json(&QmlSettings {
            tip_ms: s.tip_ms,
            poll_ms: s.poll_ms,
            guard_delay_ms: s.guard_delay_ms,
            tray_grace_ms: s.tray_grace_ms,
            page_rows: s.page_rows,
            press_frame: s.press_frame,
            press_frame_ms: s.press_frame_ms,
            frame_log: s.frame_log,
        })
    }

    /// Queues `notice` for the banner, unless the same one waits or shows.
    pub fn notice(&self, notice: Notice) {
        if self.lock().notices.push(notice) {
            self.wake(Wake::Notices);
        }
    }

    /// Hears every notice and mouse-assist note on `bus` until the subscriptions drop; the app calls it before the
    /// kernel loads.
    ///
    /// # Errors
    /// `EventError` when the bus refuses a subscription.
    pub fn listen(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        Ok(vec![
            self.hear(bus, |hub, notice: &Notice| hub.notice(notice.clone()))?,
            self.hear(bus, Self::note)?,
        ])
    }

    /// Calls `f` on the hub for every `E` on `bus`, holding the hub weakly.
    fn hear<E: Event>(
        self: &Arc<Self>,
        bus: &Bus,
        f: impl Fn(&Self, &E) + Send + Sync + 'static,
    ) -> Result<Subscription, EventError> {
        let hub = Arc::downgrade(self);
        bus.subscribe(move |event: &E| {
            if let Some(hub) = hub.upgrade() {
                f(&hub, event);
            }
        })
    }

    /// Hands a mouse-assist note to each part of the hub that follows one.
    fn note(&self, note: &AssistNote) {
        sound::on_note(self, note);
    }

    /// The waiting notices in the keyboard's language; the banner shows them from now on.
    #[must_use]
    pub fn take_notices(&self) -> Vec<Shown> {
        let mut live = self.lock();
        let Live { texts, notices, .. } = &mut *live;
        notices.take(texts.keyboard())
    }

    /// OK on shown notice `id`: it closes.
    pub fn dismiss(&self, id: u64) {
        let _ = self.lock().notices.close(id);
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
