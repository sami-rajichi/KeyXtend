//! The hub the `ui` module, the app and the QML bridges share: settings, texts, the face, the look, notices, and
//! the requests the app drains on its own thread. No lock is held while a face call, a port call or a waker runs.

mod face;
mod look;
mod notices;

pub use face::{Redraw, Resize};
pub use notices::Shown;

use crate::settings::UiSettings;
use crate::{I18n, pick};
use kx_module_api::ports::KeyboardFace;
use kx_module_api::{
    Bus, EventError, ModuleId, Notice, SettingsError, SettingsHandle, Subscription,
};
use kx_platform::ui::{Backdrop, Command, FocusToken, Screens, SysLook, SystemLook, WindowGuard};
use kx_theme::{ThemeError, Themes};
use notices::Notices;
use serde::Serialize;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use thiserror::Error;

/// The UI ports the app hands over; they run on the UI thread unless their docs say otherwise.
pub struct UiPorts {
    /// Keeps our windows from taking the focus and shapes where they take clicks.
    pub guard: Arc<dyn WindowGuard>,
    /// The screens and the pointer.
    pub screens: Arc<dyn Screens>,
    /// The system look and the UI language.
    pub look: Arc<dyn SystemLook>,
    /// The material behind a window.
    pub backdrop: Arc<dyn Backdrop>,
}

/// What the UI asks of the app, which owns the kernel and drains these on its thread.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// Try again on a failed module.
    Retry(ModuleId),
    /// The user chose Quit: stop the modules, then end the UI with `Command::Quit`.
    Quit,
}

/// What changed, so the bridges redraw on the Qt thread.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Wake {
    /// The `[ui]` settings, the texts or the face.
    Settings,
    /// The look to draw.
    Look,
    /// Notices wait for the banner.
    Notices,
    /// The app, or another copy, asks the UI to show or quit.
    Command(Command),
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

/// The timings, paging and direction QML reads, as JSON.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QmlSettings {
    tip_ms: u64,
    poll_ms: u64,
    guard_delay_ms: u64,
    page_rows: usize,
    lang: &'static str,
    rtl: bool,
}

/// What changes while the app runs.
struct Live {
    settings: UiSettings,
    i18n: Arc<I18n>,
    face: Option<Arc<dyn KeyboardFace>>,
    store: Option<SettingsHandle>,
    sys: SysLook,
    notices: Notices,
    commands: Vec<Command>,
    waker: Option<Arc<Waker>>,
}

/// The shared UI state; the app makes one and hands it to the `ui` module and the bridges.
pub struct Hub {
    ports: UiPorts,
    themes: Themes,
    /// The window that had the focus before ours, so the guard can give it back.
    focus: FocusToken,
    requests: Sender<Request>,
    live: Mutex<Live>,
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
        let (requests, drain) = mpsc::channel();
        let live = Live {
            i18n: texts(&settings, &*ports.look),
            settings,
            face: None,
            store: None,
            sys: ports.look.read(),
            notices: Notices::default(),
            commands: Vec::new(),
            waker: None,
        };
        let hub = Self {
            focus: ports.guard.remember(),
            ports,
            themes,
            requests,
            live: Mutex::new(live),
        };
        Ok((Arc::new(hub), drain))
    }

    fn lock(&self) -> MutexGuard<'_, Live> {
        self.live.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Takes the `ui` module's settings, the keyboard face and the module's own settings handle.
    pub fn attach(&self, settings: UiSettings, face: Arc<dyn KeyboardFace>, store: SettingsHandle) {
        let i18n = texts(&settings, &*self.ports.look);
        {
            let mut live = self.lock();
            live.settings = settings;
            live.i18n = i18n;
            live.face = Some(face);
            live.store = Some(store);
        }
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

    /// The texts in the UI language now.
    #[must_use]
    pub fn i18n(&self) -> Arc<I18n> {
        Arc::clone(&self.lock().i18n)
    }

    /// The timings, paging and text direction QML reads, as JSON.
    #[must_use]
    pub fn settings_json(&self) -> String {
        let live = self.lock();
        let (s, lang) = (&live.settings, live.i18n.lang());
        json(&QmlSettings {
            tip_ms: s.tip_ms,
            poll_ms: s.poll_ms,
            guard_delay_ms: s.guard_delay_ms,
            page_rows: s.page_rows,
            lang: lang.tag,
            rtl: lang.rtl,
        })
    }

    /// Registers the Qt thread's waker, replacing any earlier one, and hands it the commands that came first.
    pub fn set_waker(&self, waker: Waker) {
        let commands = {
            let mut live = self.lock();
            live.waker = Some(Arc::new(waker));
            std::mem::take(&mut live.commands)
        };
        for command in commands {
            self.wake(Wake::Command(command));
        }
    }

    /// Passes `command` from the app, or another copy, to the UI; it waits for the waker if none is set yet.
    pub fn command(&self, command: Command) {
        let waiting = {
            let mut live = self.lock();
            let none = live.waker.is_none();
            if none {
                live.commands.push(command);
            }
            none
        };
        if !waiting {
            self.wake(Wake::Command(command));
        }
    }

    /// Queues `notice` for the banner, unless the same one waits or shows.
    pub fn notice(&self, notice: Notice) {
        if self.lock().notices.push(notice) {
            self.wake(Wake::Notices);
        }
    }

    /// Hears every notice on `bus` until the subscription drops; the app calls it before the kernel loads.
    ///
    /// # Errors
    /// `EventError` when the bus refuses the subscription.
    pub fn listen(self: &Arc<Self>, bus: &Bus) -> Result<Subscription, EventError> {
        let hub = Arc::downgrade(self);
        bus.subscribe(move |notice: &Notice| {
            if let Some(hub) = hub.upgrade() {
                hub.notice(notice.clone());
            }
        })
    }

    /// The waiting notices in the UI language; the banner shows them from now on.
    #[must_use]
    pub fn take_notices(&self) -> Vec<Shown> {
        let mut live = self.lock();
        let i18n = Arc::clone(&live.i18n);
        live.notices.take(&i18n)
    }

    /// Try again on shown notice `id`: it closes, and the app restarts its module; false when it offers none.
    pub fn retry(&self, id: u64) -> bool {
        let module = self.lock().notices.close(id);
        module.is_some_and(|m| self.requests.send(Request::Retry(m)).is_ok())
    }

    /// OK on shown notice `id`: it closes.
    pub fn dismiss(&self, id: u64) {
        let _ = self.lock().notices.close(id);
    }

    /// Asks the app to quit; false when the app no longer listens.
    pub fn quit(&self) -> bool {
        self.requests.send(Request::Quit).is_ok()
    }

    /// Calls the waker, if any, outside the lock.
    fn wake(&self, what: Wake) {
        let waker = self.lock().waker.clone();
        if let Some(waker) = waker {
            waker(what);
        }
    }
}

/// The texts for `settings`' language, following the system's UI language when it says `auto`.
fn texts(settings: &UiSettings, look: &dyn SystemLook) -> Arc<I18n> {
    Arc::new(I18n::new(pick(&settings.language, &look.ui_language())))
}

/// `value` as JSON; a value that cannot be written gives `null`.
pub(crate) fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| NULL.to_owned())
}

/// JSON for nothing.
pub(crate) const NULL: &str = "null";
