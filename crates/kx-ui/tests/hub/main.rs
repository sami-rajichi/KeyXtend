//! The hub with fake ports and a fake face: settings, texts, notices, requests, face actions, the guard and the look.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod app;
#[path = "../shared/board.rs"]
mod board;
mod face;
mod fonts;
mod logs;
mod look;
mod module;
mod notices;
mod spots;
mod texts;
mod tray;

use kx_module_api::ports::{
    FaceAction, FaceError, FaceReply, KeyboardFace, KeyboardFaceKey, Point, Rect, Screen,
};
use kx_module_api::{ModuleId, Notice, NoticeArg, ServiceKey, args, keys};
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeTray, FakeWindowGuard};
use kx_test_support::host::FakeHost;
use kx_ui::{Hub, MANIFEST, Request, UiPorts, Wake};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, PoisonError};

/// A face that records every action, answers with the reply the test chose and shows a layout in the language set.
#[derive(Default)]
struct FakeFace {
    acts: Mutex<Vec<FaceAction>>,
    reply: Mutex<Option<Result<FaceReply, FaceError>>>,
    lang: Mutex<String>,
}

impl FakeFace {
    /// Every action so far, oldest first.
    fn acts(&self) -> Vec<FaceAction> {
        self.acts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Answers later actions with `reply`.
    fn answer(&self, reply: Result<FaceReply, FaceError>) {
        *self.reply.lock().unwrap_or_else(PoisonError::into_inner) = Some(reply);
    }

    /// Shows a layout in language `iso` from now on.
    fn set_lang(&self, iso: &str) {
        iso.clone_into(&mut self.lang.lock().unwrap_or_else(PoisonError::into_inner));
    }
}

impl KeyboardFace for FakeFace {
    fn board_json(&self) -> String {
        r#"{"board":1}"#.to_owned()
    }

    fn state_json(&self) -> String {
        r#"{"state":1}"#.to_owned()
    }

    fn lang(&self) -> String {
        self.lang
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn row_px(&self) -> f64 {
        f64::from(board::row())
    }

    fn act(&self, action: FaceAction) -> Result<FaceReply, FaceError> {
        self.acts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(action);
        let reply = *self.reply.lock().unwrap_or_else(PoisonError::into_inner);
        reply.unwrap_or(Ok(FaceReply::default()))
    }
}

/// The hub with its fakes, the requests it sends and the wakes it gave.
struct Rig {
    /// The hub under test.
    hub: Arc<Hub>,
    /// The requests it sends the app.
    requests: Receiver<Request>,
    /// The fake window guard.
    guard: Arc<FakeWindowGuard>,
    /// The fake screens and pointer.
    screens: Arc<FakeScreens>,
    /// The fake system look.
    look: Arc<FakeSystemLook>,
    /// The fake backdrop.
    backdrop: Arc<FakeBackdrop>,
    /// The fake tray.
    tray: Arc<FakeTray>,
    /// Every wake the hub gave.
    wakes: Arc<Mutex<Vec<Wake>>>,
}

/// The module the failure notices name.
const KEYBOARD: ModuleId = ModuleId::new("keyboard");

/// The fake tray's picture side.
const TRAY_PX: u32 = 20;

/// The kernel's notice for a failed keyboard module, which offers Try again.
fn failed() -> Notice {
    Notice {
        key: keys::MODULE_FAILED,
        module: Some(KEYBOARD),
        args: vec![(args::MODULE, NoticeArg::Text(KEYBOARD.to_string()))],
    }
}

/// Windows' 100 % DPI, the main screen's.
const DPI: u32 = 96;

/// A 100 % primary screen with a taskbar.
fn main_screen() -> Screen {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    Screen {
        rect,
        dpi: DPI,
        primary: true,
    }
}

impl Rig {
    /// A hub over fresh fakes with a waker that records.
    fn new() -> Self {
        let guard = Arc::new(FakeWindowGuard::new(0));
        let screens = Arc::new(FakeScreens::new(vec![main_screen()], Point { x: 5, y: 6 }));
        let look = Arc::new(FakeSystemLook::new());
        let backdrop = Arc::new(FakeBackdrop::new());
        let tray = Arc::new(FakeTray::new(TRAY_PX));
        let ports = UiPorts {
            guard: guard.clone(),
            screens: screens.clone(),
            look: look.clone(),
            backdrop: backdrop.clone(),
            tray: tray.clone(),
        };
        let (hub, requests) = Hub::new(ports).expect("the hub starts");
        let wakes = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&wakes);
        hub.set_waker(Box::new(move |w| seen.lock().unwrap().push(w)));
        Self {
            hub,
            requests,
            guard,
            screens,
            look,
            backdrop,
            tray,
            wakes,
        }
    }

    /// The rig's fakes as ports for another hub.
    fn ports(&self) -> UiPorts {
        UiPorts {
            guard: self.guard.clone(),
            screens: self.screens.clone(),
            look: self.look.clone(),
            backdrop: self.backdrop.clone(),
            tray: self.tray.clone(),
        }
    }

    /// The wakes so far, which it forgets.
    fn wakes(&self) -> Vec<Wake> {
        std::mem::take(&mut *self.wakes.lock().unwrap())
    }

    /// Attaches `face` with the `[ui]` defaults and a handle on `host`'s sink.
    fn attach(&self, face: Arc<FakeFace>, host: &mut FakeHost) {
        let settings = kx_ui::UiSettings::from_defaults().unwrap();
        self.hub.attach(settings, face, host.handle());
    }

    /// A rig with a fake face attached, and the host behind its settings handle.
    fn with_face() -> (Self, Arc<FakeFace>, FakeHost) {
        let rig = Self::new();
        let face = Arc::new(FakeFace::default());
        let mut host = FakeHost::new(&MANIFEST);
        rig.attach(Arc::clone(&face), &mut host);
        rig.wakes();
        (rig, face, host)
    }
}

/// Stores `face` in `host` as the keyboard face service.
fn provide(host: &mut FakeHost, face: Arc<FakeFace>) {
    let face: Arc<dyn KeyboardFace> = face;
    host.services
        .insert(KeyboardFaceKey::ID, (None, Box::new(face)));
}
